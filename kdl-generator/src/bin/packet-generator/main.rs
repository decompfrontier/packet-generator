use std::{fs::File, io::Read, path::PathBuf};

use miette::Context;
use packet_generator::{
    generators::GenerationError,
    kdl_parser::{Diagnostic, ParsingError},
};

mod cli;

#[derive(Debug, thiserror::Error)]
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
            let doc = packet_generator::kdl_parser::raw_parse_kdl(doc_str, &path)?;

            println!("Parser: {:#?}", doc);

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);

            println!("Registry: {:#?}", definitions);
        }

        cli::CliArgs::Generate { input, .. } => {
            let path = PathBuf::from(input);
            let mut file = File::open(&path).unwrap();
            let mut doc_str = String::new();
            let _ = file.read_to_string(&mut doc_str);
            let doc = packet_generator::kdl_parser::raw_parse_kdl(doc_str, &path)?;

            println!("Parser: {:#?}", doc);

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);

            let sources = packet_generator::generators::generate_glaze(&definitions)
                .map_err(|e| miette::miette!(e))
                .wrap_err("error in source code generation")?;

            for source in sources {
                println!("\n// {}\n{}\n\n", source.filename, source.content);
            }
        }
    }

    Ok(())
}
