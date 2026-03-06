#![expect(dead_code, reason = "The generators are not implemented yet.")]

use bpaf::Bpaf;

/// The tool can generate bindings for one of the following programming
/// languages. Make sure to select the one you care about.
#[derive(Debug, Clone, Bpaf)]
pub enum CxxSerializer {
    /// Serialize in JSON with Glaze
    Glaze,

    /// Serialize in JSON with simdjson
    Simdjson,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
pub struct CxxOptions {
    cxx: (),

    #[bpaf(external(cxx_serializer))]
    pub serializer: CxxSerializer,
}

/// The tool can generate bindings for one of the following programming
/// languages. Make sure to select the one you care about.
#[derive(Debug, Clone, Bpaf)]
pub enum RustSerializer {
    /// Serialize in JSON with Glaze
    Serde,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
pub struct RustOptions {
    rust: (),

    #[bpaf(external(rust_serializer))]
    pub serializer: RustSerializer,
}

/// The tool can generate bindings for one of the following programming
/// languages.
///
/// Make sure to select the one you care about.
#[derive(Debug, Clone, Bpaf)]
pub enum ProgrammingLanguage {
    Cxx(#[bpaf(external(cxx_options))] CxxOptions),

    /// Rust
    Rust(#[bpaf(external(rust_options))] RustOptions),
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum CliArgs {
    /// Parses a KDL file and dumps its representation, without generating
    /// anything else.
    #[bpaf(command("dump"))]
    DumpRepresentation {
        /// KDL definition file to parse.
        /// May also be a directory.
        ///
        /// Example: mst/unit.kdl
        #[bpaf(positional("INPUT"))]
        input: String,
    },

    /// Parses a KDL file and generates the boilerplate code for a language.
    #[bpaf(command("generate"))]
    Generate {
        /// KDL definition file to parse.
        ///
        /// Example: mst/unit.kdl
        #[bpaf(short, long)]
        input: String,

        /// The directory where to generate implementations for the definitions
        /// provided in FILENAME.
        #[bpaf(short, long)]
        output_directory: Option<String>,

        /// The language for which the tool should generate bindings for.
        #[bpaf(external(programming_language))]
        language: ProgrammingLanguage,
    },

    /// Prints version information.
    #[bpaf(command("version"))]
    Version {},
}

pub mod commands {
    use bpaf::Bpaf;

    #[derive(Debug, Clone, Bpaf)]
    pub struct Generate {
        /// KDL definition file to parse.
        ///
        /// Example: mst/unit.kdl
        #[bpaf(positional("FILENAME"))]
        pub filename: String,

        /// The directory were to generate implementations for the definitions
        /// provided in FILENAME.
        #[bpaf(short, long)]
        pub output_directory: String,
    }
}

pub fn parse_args() -> CliArgs {
    cli_args().run()
}
