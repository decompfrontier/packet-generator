#![allow(clippy::expect_used, reason = "Sir, this is a benchmark")]
#![allow(clippy::unwrap_used, reason = "Sir, this is a benchmark")]

use criterion::{Criterion, criterion_group, criterion_main};
use packet_generator::{
    intermediate::DefinitionRegistry,
    kdl_parser::{ParserOpts, ParsingError, ParsingWarnings, UnparsedKdl},
    vfs::{InMemoryFS, VfsPath},
};
use std::path::PathBuf;

fn parse_files(
    unparsed_kdls: &[UnparsedKdl<'_>],
    parser_opts: &ParserOpts<InMemoryFS>,
) -> Result<(DefinitionRegistry, ParsingWarnings), ParsingError> {
    let res = packet_generator::parse_kdl(unparsed_kdls, parser_opts);

    assert!(
        res.is_ok(),
        "parser failed with reason: {}",
        res.unwrap_err()
    );

    res
}

fn add_all_paths(prefix: &str, directory: &str, fs: &mut InMemoryFS) {
    let mst_paths =
        glob::glob(&format!("{prefix}/{directory}/**/*.kdl")).expect("pattern is correct");

    for path in mst_paths {
        let path = path.expect("FS works");
        let content = std::fs::read_to_string(&path).expect("FS works");

        let vfs_path = path.strip_prefix(prefix).expect("can remove assets prefix");

        let _ = fs.add_file(VfsPath::new(vfs_path), &content);
    }
}

fn build_stresstest_input() -> (&'static str, PathBuf, ParserOpts<InMemoryFS>) {
    let mut fs = InMemoryFS::new();

    let main_content = include_str!("../tests/defs/main.kdl");

    add_all_paths("tests/defs", "", &mut fs);

    let opts = ParserOpts::new(fs);

    (main_content, PathBuf::from("main.kdl"), opts)
}

fn build_gamefrontier_input() -> (&'static str, PathBuf, ParserOpts<InMemoryFS>) {
    let mut fs = InMemoryFS::new();

    let main_content = include_str!("../assets/all.kdl");

    add_all_paths("assets", "mst", &mut fs);
    add_all_paths("assets", "net", &mut fs);

    let _ = fs.add_file(VfsPath::new("all.kdl"), main_content);

    let opts = ParserOpts::new(fs);

    (main_content, PathBuf::from("all.kdl"), opts)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    {
        let (main_content, path, opts) = build_stresstest_input();
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter("stress-test files"),
            &(main_content, path, &opts),
            |b, input| {
                let unparsed_kdl = UnparsedKdl::new(input.0, &input.1);
                let files = &[unparsed_kdl];
                b.iter_with_large_drop(|| parse_files(files, input.2));
            },
        );
    }

    {
        let (main_content, path, opts) = build_gamefrontier_input();
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter("Brave Frontier files"),
            &(main_content, path, &opts),
            |b, input| {
                let unparsed_kdl = UnparsedKdl::new(input.0, &input.1);
                let files = &[unparsed_kdl];
                b.iter_with_large_drop(|| parse_files(files, input.2));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
