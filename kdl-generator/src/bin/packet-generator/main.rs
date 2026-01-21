#![forbid(clippy::unwrap_used, clippy::unwrap_in_result)]

use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use miette::Context;
use packet_generator::{
    generators::{
        self, CxxGenerator, GenerationError, Generator, GlazeGenerator, PrimaryGenerator,
        SecondaryGenerator, WithAddons,
    },
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
            let mut file = File::open(&path).unwrap();
            let mut doc_str = String::new();
            let _ = file.read_to_string(&mut doc_str);
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
            let mut file = File::open(&path)
                .map_err(|e| miette::miette!(e))
                .wrap_err(format!("Unable to open source file: {input}"))?;

            let mut doc_str = String::new();
            let _ = file.read_to_string(&mut doc_str);
            let (doc, warnings) = packet_generator::kdl_parser::raw_parse_kdl(
                doc_str,
                &path,
                &ParserOpts::default(),
            )?;

            warnings.print_warnings_if_any();

            println!("Parser: {:#?}", doc);

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);

            /*
            let mut source_output = String::new();

            for ele in &generators {
                source_output.push_str(ele.get_prefix().as_str());
                source_output.push('\n');
            }

            source_output.push('\n');

            for ele in &generators {
                let content = ele
                    .step(&definitions)
                    .map_err(|e| miette::miette!(e))
                    .wrap_err("error in source code generation")?;

                source_output.push_str(content.as_str());
                source_output.push('\n');
            }

            source_output.push('\n');

            for ele in &generators {
                source_output.push_str(ele.get_suffix().as_str());
                source_output.push('\n');
            }
            */

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
