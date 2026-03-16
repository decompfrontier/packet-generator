#![allow(clippy::expect_used, reason = "Sir, this is a benchmark")]
#![allow(clippy::unwrap_used, reason = "Sir, this is a benchmark")]

use criterion::{Criterion, criterion_group, criterion_main};
use packet_generator::{
    intermediate::DefinitionRegistry,
    kdl_parser::{ParserOpts, ParsingError, ParsingWarnings},
    vfs::{InMemoryFS, VfsPath},
};
use std::path::PathBuf;

fn parse_files(
    main_content: &'static str,
    path: &PathBuf,
    parser_opts: &ParserOpts<InMemoryFS>,
) -> Result<(DefinitionRegistry, ParsingWarnings), ParsingError> {
    let res = packet_generator::parse_kdl(main_content, path, parser_opts);

    assert!(
        res.is_ok(),
        "parser failed with reason: {}",
        res.unwrap_err()
    );

    res
}

fn build_stresstest_input() -> (&'static str, PathBuf, ParserOpts<InMemoryFS>) {
    let mut fs = InMemoryFS::new();

    let main_content = include_str!("../tests/defs/main.kdl");

    let _ = fs.add_file(VfsPath::new("main.kdl"), main_content);

    macro_rules! add_path {
        ($name:expr) => {
            let _ = fs.add_file(
                VfsPath::new($name),
                include_str!(concat!("../tests/defs/", $name)),
            );
        };
    }

    add_path!("http.kdl");
    add_path!("plist.kdl");
    add_path!("xml.kdl");
    add_path!("empty.kdl");
    add_path!("included-2.kdl");
    add_path!("nested/included-3.kdl");

    let opts = ParserOpts::new(fs);

    (main_content, PathBuf::from("main.kdl"), opts)
}

fn build_gamefrontier_input() -> (&'static str, PathBuf, ParserOpts<InMemoryFS>) {
    let mut fs = InMemoryFS::new();

    let main_content = include_str!("../assets/all.kdl");

    macro_rules! add_path {
        ($name:expr) => {
            let _ = fs.add_file(
                VfsPath::new($name),
                include_str!(concat!("../assets/", $name)),
            );
        };
    }

    let _ = fs.add_file(VfsPath::new("all.kdl"), main_content);

    add_path!("mst/arena.kdl");
    add_path!("mst/banner.kdl");
    add_path!("mst/daily_task.kdl");

    add_path!("mst/define.kdl");

    add_path!("mst/dungeon_key.kdl");
    add_path!("mst/event.kdl");
    add_path!("mst/excluded_dungeon.kdl");
    add_path!("mst/first_desc.kdl");
    add_path!("mst/gacha.kdl");
    add_path!("mst/gift.kdl");
    add_path!("mst/login_campaign.kdl");
    add_path!("mst/notice_info.kdl");
    add_path!("mst/info.kdl");
    add_path!("mst/trophy.kdl");
    add_path!("mst/help.kdl");
    add_path!("mst/frontier_hunter.kdl");
    add_path!("mst/npc.kdl");
    add_path!("mst/passive_skill.kdl");
    add_path!("mst/receipe.kdl");
    add_path!("mst/slots.kdl");
    add_path!("mst/town.kdl");
    add_path!("mst/unit.kdl");
    add_path!("mst/url.kdl");
    add_path!("mst/user_level.kdl");
    add_path!("mst/version_info.kdl");
    add_path!("mst/sound.kdl");

    add_path!("net/badge_info.kdl");
    add_path!("net/challenge_arena.kdl");
    add_path!("net/daily_login.kdl");
    add_path!("net/feature_check.kdl");
    add_path!("net/featuring_gate.kdl");
    add_path!("net/friends.kdl");
    add_path!("net/gme.kdl");
    add_path!("net/guild.kdl");
    add_path!("net/gumi_live.kdl");

    add_path!("net/handlers.kdl");
    add_path!("net/items.kdl");
    add_path!("net/mission.kdl");
    add_path!("net/npc_message_overwrite.kdl");
    add_path!("net/permit_place.kdl");
    add_path!("net/raid.kdl");
    add_path!("net/reinforcement_info.kdl");
    add_path!("net/signal_key.kdl");
    add_path!("net/summoner_journal.kdl");
    add_path!("net/user.kdl");

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
            |b, input| b.iter_with_large_drop(|| parse_files(input.0, &input.1, input.2)),
        );
    }

    {
        let (main_content, path, opts) = build_gamefrontier_input();
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter("Brave Frontier files"),
            &(main_content, path, &opts),
            |b, input| b.iter_with_large_drop(|| parse_files(input.0, &input.1, input.2)),
        );
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
