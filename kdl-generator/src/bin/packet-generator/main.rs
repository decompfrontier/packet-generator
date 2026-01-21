#![forbid(clippy::unwrap_used, clippy::unwrap_in_result)]

use std::path::PathBuf;

use miette::Context;
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

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);

            println!("Registry: {:#?}", definitions);
        }

        cli::CliArgs::Generate {
            input,
            language,
            output_directory: _,
        } => {
            // let primary: Box<dyn PrimaryGenerator>;

            let mut generators: Vec<Box<dyn Generator>> = vec![];

            match language {
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

                    generators.push(Box::new(cxx_generator));
                }

                cli::ProgrammingLanguage::Rust(_serializer) => {
                    return Err(miette::miette!(
                        "Rust generator is currently not implemented!"
                    ));
                }
            }

            let path = PathBuf::from(input.clone());

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

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);

            for generator in &generators {
                let source_output = generator
                    .generate(&definitions, "test")
                    .map_err(|e| miette::miette!(e))?;

                println!("{}\n", source_output.content)
            }
        }
    }

    Ok(())
}
