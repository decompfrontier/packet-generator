#![forbid(clippy::unwrap_used, clippy::unwrap_in_result)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{env::current_dir, path::PathBuf};

use miette::{Context, miette};
use packet_generator::generators::write_sources;
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

            let doc_str = std::fs::read_to_string(&path)
                .map_err(|e| miette::miette!(e))
                .wrap_err_with(|| format!("cannot open file: {}", path.display()))?;

            let (doc, warnings) = packet_generator::kdl_parser::raw_parse_kdl(
                doc_str,
                &path,
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

            let input_path = PathBuf::from(input);

            let doc_str = std::fs::read_to_string(&input_path)
                .map_err(|e| miette::miette!(e))
                .wrap_err_with(|| format!("cannot open file: {}", input_path.display()))?;

            let (doc, warnings) = packet_generator::kdl_parser::raw_parse_kdl(
                doc_str,
                &input_path,
                &ParserOpts::default(),
            )?;

            warnings.print_warnings_if_any();

            let doc = doc.finalize()?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc)?;

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
                .map_err(|e| miette::miette!("could not generate sources: {e}"))?;

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
