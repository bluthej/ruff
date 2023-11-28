use itertools::Itertools;

use super::settings::Settings;
use super::sorting::{MemberKey, ModuleKey};
use super::types::EitherImport::{self, Import, ImportFrom};
use super::types::{AliasData, CommentSet, ImportBlock, ImportFromStatement};

pub(crate) fn order_imports<'a>(
    block: ImportBlock<'a>,
    settings: &Settings,
) -> Vec<EitherImport<'a>> {
    let straight_imports = block.import.into_iter();

    let from_imports =
        // Include all non-re-exports.
        block
            .import_from
            .into_iter()
            .chain(
                // Include all re-exports.
                block
                    .import_from_as
                    .into_iter()
                    .map(|((import_from, ..), body)| (import_from, body)),
            )
            .chain(
                // Include all star imports.
                block.import_from_star,
            )
            .map(
                |(
                    import_from,
                    ImportFromStatement {
                        comments,
                        aliases,
                        trailing_comma,
                    },
                )| {
                    // Within each `Stmt::ImportFrom`, sort the members.
                    (
                        import_from,
                        comments,
                        trailing_comma,
                        aliases
                            .into_iter()
                            .sorted_by_cached_key(|(alias, _)| {
                                MemberKey::from_member(alias.name, alias.asname, settings)
                            })
                            .collect::<Vec<(AliasData, CommentSet)>>(),
                    )
                },
            );

    let ordered_imports = if settings.force_sort_within_sections {
        straight_imports
            .map(Import)
            .chain(from_imports.map(ImportFrom))
            .sorted_by_cached_key(|import| ModuleKey::from_either_import(import, settings))
            .collect()
    } else {
        let ordered_straight_imports = straight_imports
            .sorted_by_cached_key(|import| ModuleKey::from_straight_import(import, settings));
        let ordered_from_imports = from_imports
            .sorted_by_cached_key(|import| ModuleKey::from_from_import(import, settings));
        if settings.from_first {
            ordered_from_imports
                .into_iter()
                .map(ImportFrom)
                .chain(ordered_straight_imports.into_iter().map(Import))
                .collect()
        } else {
            ordered_straight_imports
                .into_iter()
                .map(Import)
                .chain(ordered_from_imports.into_iter().map(ImportFrom))
                .collect()
        }
    };

    ordered_imports
}
