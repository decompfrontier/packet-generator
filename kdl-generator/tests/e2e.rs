use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
};

use packet_generator::{
    generators,
    intermediate::DefinitionRegistry,
    kdl_parser::{ParserOpts, ParsingWarnings},
};

const CXX_COMPILER: &str = "clang++";
const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn setup_e2e_registry(main_file: &str) -> (DefinitionRegistry, ParsingWarnings) {
    let path = PathBuf::from(format!("{PROJECT_DIR}/{main_file}"));
    let document = std::fs::read_to_string(&path).unwrap();
    packet_generator::parse_kdl(&document, &path, &ParserOpts::default())
        .map_err(|e| miette::Report::from(e))
        .unwrap()
}

#[test]
fn e2e_can_compile_cxx_definition() {
    let (defs, _) = setup_e2e_registry("tests/defs/main.kdl");
    let cxx_generated = generators::generate_cxx(&defs).unwrap();

    let generation_basepath =
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/target/tests-e2e-cxx"));
    let _ = std::fs::create_dir_all(&generation_basepath);
    let new_file_path = generation_basepath.join(&cxx_generated.filename);
    let mut new_file = File::create(&new_file_path).unwrap();

    new_file
        .write_all(&cxx_generated.content.as_bytes())
        .unwrap();

    let args = &[
        "-isystem",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../runtime/cpp"),
        new_file_path.to_str().unwrap(),
    ];

    let clang_output = std::process::Command::new(CXX_COMPILER)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    let stdout = str::from_utf8(&clang_output.stdout).unwrap();
    let stderr = str::from_utf8(&clang_output.stderr).unwrap();

    println!("{CXX_COMPILER} args: {CXX_COMPILER} {}", args.join(" "));
    println!("{CXX_COMPILER} stdout:\n{stdout}");
    println!("{CXX_COMPILER} stderr:\n{stderr}");

    assert!(clang_output.status.success());

    println!("clang++ output: {clang_output:#?}");
}

#[test]
fn e2e_can_compile_glaze_definition() {
    let (defs, _) = setup_e2e_registry("tests/defs/main.kdl");
    let cxx_generated = generators::generate_glaze(&defs).unwrap();

    let generation_basepath =
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/target/tests-e2e-glz"));
    let _ = std::fs::create_dir_all(&generation_basepath);
    let new_file_path = generation_basepath.join(&cxx_generated.filename);
    let mut new_file = File::create(&new_file_path).unwrap();

    new_file
        .write_all(&cxx_generated.content.as_bytes())
        .unwrap();

    let args = &[
        "-isystem",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../runtime/cpp"),
        new_file_path.to_str().unwrap(),
    ];

    let clang_output = std::process::Command::new(CXX_COMPILER)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    let stdout = str::from_utf8(&clang_output.stdout).unwrap();
    let stderr = str::from_utf8(&clang_output.stderr).unwrap();

    println!("{CXX_COMPILER} args: {CXX_COMPILER} {}", args.join(" "));
    println!("{CXX_COMPILER} stdout:\n{stdout}");
    println!("{CXX_COMPILER} stderr:\n{stderr}");

    assert!(clang_output.status.success());

    println!("clang++ output: {clang_output:#?}");
}

#[test]
fn e2e_can_parse_assets() {
    let (registry, _) = setup_e2e_registry("assets/mst/receipe.kdl");

    println!("{registry:#?}");
}
