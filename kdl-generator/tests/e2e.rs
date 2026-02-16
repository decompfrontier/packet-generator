use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
};

use packet_generator::{
    generators::{self, Generator, GlazeGenerator, WithAddons},
    intermediate::DefinitionRegistry,
    kdl_parser::{ParserOpts, ParsingWarnings},
};
use stringcase::Caser;

const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn setup_e2e_registry(main_file: &str) -> (DefinitionRegistry, ParsingWarnings) {
    let path = PathBuf::from_iter([PROJECT_DIR, main_file]);
    let document = std::fs::read_to_string(&path).expect("this is a test, we control the string");
    packet_generator::parse_kdl(&document, &path, &ParserOpts::default())
        .map_err(miette::Report::from)
        .expect("we control the files")
}

fn create_file(path: impl AsRef<Path>, content: &str) {
    let mut f = File::create(path).expect("FS must create files");
    f.write_all(content.as_bytes())
        .expect("FS must write bytes");
}

fn generic_e2e_cxx_glaze_harness(path_entrypoint: &str, test_name: &str) {
    let (defs, _) = setup_e2e_registry(path_entrypoint);

    let generation_basepath = PathBuf::from_iter([
        env!("CARGO_MANIFEST_DIR"),
        test_name,
        "target",
    ]);

    let _ = std::fs::create_dir_all(&generation_basepath);

    let mut generator = generators::CxxGenerator::new();
    generator.add_addon(GlazeGenerator {});

    let source = generator
        .generate(&defs, "main")
        .expect("should generate good code");

    create_file(generation_basepath.join(&source.filename), &source.content);

    {
        // this should be enough as cmake should be clever
        let runtime_dir = env!("CARGO_MANIFEST_DIR").replace("\\", "/") + "/../runtime/cpp";
        let test_name = test_name.to_snake_case();
        let cmake_file_content = format!(
            r#"
cmake_minimum_required(VERSION 3.24)
project(kdl_generator_e2e_{test_name}_test_suite)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)

FetchContent_Declare(
  glaze
  GIT_REPOSITORY https://github.com/stephenberry/glaze.git
  GIT_TAG main
  GIT_SHALLOW TRUE
)

FetchContent_MakeAvailable(glaze)

if (MSVC)
    set(CMAKE_CXX_FLAGS "${{CMAKE_CXX_FLAGS}} /MP /utf-8 /W3 /permissive-")
else()
    set(CMAKE_CXX_FLAGS "${{CMAKE_CXX_FLAGS}} -Wall -fno-permissive")
endif()

add_executable(${{PROJECT_NAME}}
    main.cpp
)
target_include_directories(${{PROJECT_NAME}} PRIVATE "{runtime_dir}")
target_link_libraries(${{PROJECT_NAME}} PRIVATE glaze::glaze)
    "#
        );

        create_file(
            generation_basepath.join("CMakeLists.txt"),
            &cmake_file_content,
        );
    }

    {
        let main_cpp_content = format!(
            r#"
#include "{}"

int main() {{
    // Intentionally empty, let the compiler check the headers.
}}"#,
            source.filename,
        );

        create_file(generation_basepath.join("main.cpp"), &main_cpp_content);
    }

    {
        let cmake_args = &["-S", ".", "-B", "build/"];
        let cmake_output = std::process::Command::new("cmake")
            .current_dir(&generation_basepath)
            .args(cmake_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("the OS can spawn processes");

        let stdout = str::from_utf8(&cmake_output.stdout).expect("string is UTF-8");
        let stderr = str::from_utf8(&cmake_output.stderr).expect("string is UTF-8");

        println!("CMake stdout:\n{stdout}");
        println!("CMake stderr:\n{stderr}");

        assert!(cmake_output.status.success());
    }

    {
        let cmake_args = &["--build", "build"];
        let cmake_output = std::process::Command::new("cmake")
            .current_dir(&generation_basepath)
            .args(cmake_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("the OS can spawn processes");

        let stdout = str::from_utf8(&cmake_output.stdout).expect("string is UTF-8");
        let stderr = str::from_utf8(&cmake_output.stderr).expect("string is UTF-8");

        println!("CMake stdout:\n{stdout}");
        println!("CMake stderr:\n{stderr}");

        assert!(cmake_output.status.success());
    }
}

#[test]
fn e2e_can_compile_cxx_glaze_stresstest() {
    generic_e2e_cxx_glaze_harness("tests/defs/main.kdl", "generic-definitions");
}

#[test]
fn e2e_can_compile_cxx_glaze_brave_frontier() {
    generic_e2e_cxx_glaze_harness("assets/net/handlers.kdl", "brave-frontier");
}
