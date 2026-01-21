#![forbid(clippy::unwrap_used, clippy::unwrap_in_result)]

use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use miette::Context;
use packet_generator::{
    generators::{self, GenerationError, PrimaryGenerator, SecondaryGenerator},
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

            if warnings.are_there_any() {
                let warnings = miette::ErrReport::from(warnings);
                println!("{warnings:?}");
            }

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
            let primary: Arc<dyn PrimaryGenerator>;
            let mut generators: Vec<Arc<dyn SecondaryGenerator>> = Vec::new();

            match language {
                cli::ProgrammingLanguage::Cxx(options) => {
                    primary = Arc::new(generators::CxxGenerator {});
                    generators.push(primary);
                    match options.serializer {
                        CxxSerializer::Glaze => {
                            generators.push(Arc::new(generators::GlazeGenerator {}));
                        }
                        CxxSerializer::Simdjson => {
                            return Err(miette::miette!(
                                "Simdjson secondary generator for Cxx is not implemented!"
                            ));
                        }
                    }
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

            if warnings.are_there_any() {
                let warnings = miette::Report::from(warnings);
                println!("{warnings:?}");
            }

            println!("Parser: {:#?}", doc);

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);
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

            println!("{}\n", source_output)
        }
    }

    Ok(())
}
