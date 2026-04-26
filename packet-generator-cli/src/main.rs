#![forbid(clippy::unwrap_used, clippy::unwrap_in_result)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{env::current_dir, path::PathBuf};

use itertools::Itertools;
use miette::{Context, IntoDiagnostic, miette};
use packet_generator::generators::write_sources;
use packet_generator::kdl_parser::UnparsedKdl;
use packet_generator::{
    generators::{CxxGenerator, GenerationError, Generator, GlazeGenerator, WithAddons},
    kdl_parser::{Diagnostic, ParserOpts, ParsingError},
};

use crate::cli::CxxSerializer;

mod cli;

#[derive(Debug, thiserror::Error)]
#[expect(dead_code)]
enum ApplicationError {
    #[error(transparent)]
    MietteReport(#[from] ParsingError),

    #[error(transparent)]
    Diagnostic(#[from] Diagnostic),

    #[error(transparent)]
    Generation(#[from] GenerationError),
}

fn main() -> Result<(), miette::Report> {
    let args = cli::parse_args();

    match args {
        cli::CliArgs::DumpRepresentation { input } => {
            let path = PathBuf::from(input);

            let kdl_document_content = std::fs::read_to_string(&path)
                .map_err(|e| miette::miette!(e))
                .wrap_err_with(|| format!("cannot open file: {}", path.display()))?;

            let unparsed_document = UnparsedKdl::new(&kdl_document_content, &path);

            let (doc, warnings) = packet_generator::kdl_parser::raw_parse_kdl(
                &[unparsed_document],
                &ParserOpts::default(),
            )?;

            warnings.print_warnings_if_any();

            println!("Parser: {:#?}", doc);

            let doc = doc.finalize()?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc)?;

            println!("Registry: {:#?}", definitions);
        }

        cli::CliArgs::Generate {
            input,
            language,
            output_directory,
        } => {
            let output_directory = match output_directory.map(PathBuf::from) {
                Some(v) => v,
                None => current_dir().map_err(|e| miette::miette!(e))?,
            };

            let input_path = PathBuf::from(&input);

            let (doc, warnings) = if input_path.is_dir() {
                let files_to_read = glob::glob(&format!("{input}/**/*.kdl"))
                    .into_diagnostic()
                    .wrap_err_with(|| format!("error creating glob pattern for '{}'", input))?;

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

                packet_generator::kdl_parser::raw_parse_kdl(&paths, &ParserOpts::default())?
            } else {
                let kdl_document_content = std::fs::read_to_string(&input_path)
                    .map_err(|e| miette::miette!(e))
                    .wrap_err_with(|| format!("cannot open file: {}", input_path.display()))?;

                let unparsed_document = UnparsedKdl::new(&kdl_document_content, &input_path);
                packet_generator::kdl_parser::raw_parse_kdl(
                    &[unparsed_document],
                    &ParserOpts::default(),
                )?
            };

            warnings.print_warnings_if_any();

            let doc = doc.finalize()?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc)?;

            let generator = match language {
                cli::ProgrammingLanguage::Cxx(options) => {
                    let mut cxx_generator = CxxGenerator::new();

                    match options.serializer {
                        CxxSerializer::Glaze => {
                            cxx_generator.add_addon(GlazeGenerator {});
                        }

                        CxxSerializer::Simdjson => {
                            return Err(miette::miette!(
                                "Simdjson secondary generator for Cxx is not implemented!"
                            ));
                        }
                    }

                    Box::new(cxx_generator)
                }

                cli::ProgrammingLanguage::Rust(_serializer) => {
                    return Err(miette::miette!(
                        "Rust generator is currently not implemented!"
                    ));
                }
            };

            let sources = generator
                .generate(
                    &definitions,
                    input_path
                        .file_stem()
                        .ok_or_else(|| {
                            miette!(
                                "could not obtain file name from path `{}`",
                                input_path.display()
                            )
                        })?
                        .to_str()
                        .ok_or_else(|| {
                            miette!(
                                "file name `{}` is not a valid UTF-8 string",
                                input_path.display()
                            )
                        })?,
                )
                .into_diagnostic()
                .wrap_err("could not generate sources")?;

            write_sources(&output_directory, &sources)?;
        }

        cli::CliArgs::Version {} => {
            let version = env!("CARGO_PKG_VERSION");
            print!("packet-generator v{version} ");

            #[cfg(feature = "mimalloc")]
            {
                print!("[+mimalloc] ");
            }

            println!();
        }
    }

    Ok(())
}
