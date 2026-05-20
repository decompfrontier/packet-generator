use std::path::Path;

use itertools::Itertools;
use miette::{Context, IntoDiagnostic};
use packet_generator::kdl_parser::{ParserOpts, ParsingWarnings, UnparsedKdl, schema::RawDocument};

/// Reads KDL in a directory for the CLI.
///
/// # Errors
///
/// Errors if we cannot read the files from the filesystem.
pub fn read_all_kdls_from_directory(
    input: &Path,
) -> Result<(RawDocument, ParsingWarnings), miette::Report> {
    let files_to_read = glob::glob(&format!("{}/**/*.kdl", input.display()))
        .into_diagnostic()
        .wrap_err_with(|| format!("error creating glob pattern for '{}'", input.display()))?;

    let paths = files_to_read
        .process_results(|maybe_paths| {
            maybe_paths
                .map(|p| -> Result<_, miette::Report> {
                    let kdl_document_content = std::fs::read_to_string(&p)
                        .into_diagnostic()
                        .wrap_err("cannot read file")?;

                    Ok(UnparsedKdl::new_owned(kdl_document_content, p))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .into_diagnostic()
        .wrap_err("cannot read globs")??;

    Ok(packet_generator::kdl_parser::raw_parse_kdl(
        &paths,
        &ParserOpts::default(),
    )?)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic_in_result_fn, reason = "Sir, this is a test")]
    use packet_generator::intermediate::schema::Definition;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn read_directory_works() -> Result<(), miette::Report> {
        let assets_dir = PathBuf::from_iter(&[env!("CARGO_MANIFEST_DIR"), "tests", "assets"]);

        dbg!(&assets_dir);

        let (document, warnings) = read_all_kdls_from_directory(&assets_dir)?;

        if !warnings.are_there_any() {
            warnings.print_warnings_if_any();
            miette::bail!("There were warnings");
        }

        let document = document.finalize()?;

        let definitions = packet_generator::kdl_parser::document_to_definitions(document)?;

        dbg!(&definitions);

        let (foo, _) = definitions.find("Foo").wrap_err("Cannot find Foo")?;
        let (bar, _) = definitions.find("Bar").wrap_err("Cannot find Bar")?;
        let (baz, _) = definitions.find("Baz").wrap_err("Cannot find Baz")?;

        assert_eq!(foo.name(), "Foo");
        if let Definition::Json(j) = foo {
            assert_eq!(j.fields.len(), 1);
            assert!(j.fields.contains("bar"));
        } else {
            miette::bail!("Foo is not a JSON!")
        }

        assert_eq!(bar.name(), "Bar");
        if let Definition::Json(j) = bar {
            assert_eq!(j.fields.len(), 1);
            assert!(j.fields.contains("baz"));
        } else {
            miette::bail!("Bar is not a JSON!")
        }

        assert_eq!(baz.name(), "Baz");
        if let Definition::StringEnum(j) = baz {
            assert_eq!(j.variants.len(), 1);
            assert!(j.variants.contains("test"));
        } else {
            miette::bail!("Baz is not a string enum!")
        }

        Ok(())
    }
}
