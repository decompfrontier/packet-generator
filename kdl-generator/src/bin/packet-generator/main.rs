use facet_pretty::FacetPretty;
use std::{fs::File, io::Read};

mod cli;

fn main() -> Result<(), miette::Report> {
    let args = cli::parse_args();

    #[expect(clippy::single_match, reason = "::Generate not implemented yet.")]
    match args {
        cli::CliArgs::DumpRepresentation { input } => {
            let mut file = File::open(input).unwrap();
            let mut doc_str = String::new();
            let _ = file.read_to_string(&mut doc_str);
            let doc = packet_generator::kdl_parser::raw_parse_kdl(doc_str)
                .map_err(miette::Report::new)?;
            println!("{}", doc.pretty());

            let doc = packet_generator::kdl_parser::validate(doc)?;

            let definitions = packet_generator::kdl_parser::document_to_definitions(doc);

            println!("{:#?}", definitions);
        }

        _ => {} // cli::CliArgs::Generate {
                //     input,
                //     output_directory: _
                // } => {
                //     let mut file = File::open(input).unwrap();
                //     let mut doc_str = String::new();
                //     let _ = file.read_to_string(&mut doc_str);
                //     let doc: Document = kdl::from_str(&doc_str)?;
                //     println!("{}", doc.pretty());
                // }
    }

    Ok(())
}
