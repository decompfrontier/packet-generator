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

    let _ = fs.add_file(
        VfsPath::new("http.kdl"),
        include_str!("../tests/defs/http.kdl"),
    );

    let _ = fs.add_file(
        VfsPath::new("plist.kdl"),
        include_str!("../tests/defs/plist.kdl"),
    );

    let _ = fs.add_file(
        VfsPath::new("xml.kdl"),
        include_str!("../tests/defs/xml.kdl"),
    );

    let _ = fs.add_file(
        VfsPath::new("empty.kdl"),
        include_str!("../tests/defs/empty.kdl"),
    );

    let _ = fs.add_file(
        VfsPath::new("included-2.kdl"),
        include_str!("../tests/defs/included-2.kdl"),
    );

    let _ = fs.add_file(
        VfsPath::new("nested/included-3.kdl"),
        include_str!("../tests/defs/nested/included-3.kdl"),
    );

    let opts = ParserOpts::new(fs);

    (main_content, PathBuf::from("main.kdl"), opts)
}

fn build_gamefrontier_input() -> (&'static str, PathBuf, ParserOpts<InMemoryFS>) {
    let mut fs = InMemoryFS::new();

    let main_content = include_str!("../assets/net/handlers.kdl");

    let _ = fs.add_file(
        VfsPath::new("mst/arena.kdl"),
        include_str!("../assets/mst/arena.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/banner.kdl"),
        include_str!("../assets/mst/banner.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/dailytaskbonus.kdl"),
        include_str!("../assets/mst/dailytaskbonus.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/define.kdl"),
        include_str!("../assets/mst/define.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/dungeonkey.kdl"),
        include_str!("../assets/mst/dungeonkey.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/events.kdl"),
        include_str!("../assets/mst/events.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/excludeddungeon.kdl"),
        include_str!("../assets/mst/excludeddungeon.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/firstdesc.kdl"),
        include_str!("../assets/mst/firstdesc.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/gatcha.kdl"),
        include_str!("../assets/mst/gatcha.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/gift.kdl"),
        include_str!("../assets/mst/gift.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/logincampaign.kdl"),
        include_str!("../assets/mst/logincampaign.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/noticeinfo.kdl"),
        include_str!("../assets/mst/noticeinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/npc.kdl"),
        include_str!("../assets/mst/npc.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/passiveskill.kdl"),
        include_str!("../assets/mst/passiveskill.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/receipe.kdl"),
        include_str!("../assets/mst/receipe.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/slots.kdl"),
        include_str!("../assets/mst/slots.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/town.kdl"),
        include_str!("../assets/mst/town.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/unit.kdl"),
        include_str!("../assets/mst/unit.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/url.kdl"),
        include_str!("../assets/mst/url.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/userlevel.kdl"),
        include_str!("../assets/mst/userlevel.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("mst/versioninfo.kdl"),
        include_str!("../assets/mst/versioninfo.kdl"),
    );

    let _ = fs.add_file(
        VfsPath::new("net/badgeinfo.kdl"),
        include_str!("../assets/net/badgeinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/challenge_arena.kdl"),
        include_str!("../assets/net/challenge_arena.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/daily_login.kdl"),
        include_str!("../assets/net/daily_login.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/featurecheck.kdl"),
        include_str!("../assets/net/featurecheck.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/featuring_gate.kdl"),
        include_str!("../assets/net/featuring_gate.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/friends.kdl"),
        include_str!("../assets/net/friends.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/gme.kdl"),
        include_str!("../assets/net/gme.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/guild.kdl"),
        include_str!("../assets/net/guild.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/gumi_live.kdl"),
        include_str!("../assets/net/gumi_live.kdl"),
    );
    let _ = fs.add_file(VfsPath::new("net/handlers.kdl"), main_content);
    let _ = fs.add_file(
        VfsPath::new("net/items.kdl"),
        include_str!("../assets/net/items.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/logincampaign.kdl"),
        include_str!("../assets/net/logincampaign.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/missionbreakinfo.kdl"),
        include_str!("../assets/net/missionbreakinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/npcmessageoverwriteinfo.kdl"),
        include_str!("../assets/net/npcmessageoverwriteinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/permitplace.kdl"),
        include_str!("../assets/net/permitplace.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/raid.kdl"),
        include_str!("../assets/net/raid.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/reinforcementinfo.kdl"),
        include_str!("../assets/net/reinforcementinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/signalkey.kdl"),
        include_str!("../assets/net/signalkey.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/summonerjournaluserinfo.kdl"),
        include_str!("../assets/net/summonerjournaluserinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/summonticketuserinfo.kdl"),
        include_str!("../assets/net/summonticketuserinfo.kdl"),
    );
    let _ = fs.add_file(
        VfsPath::new("net/userinfo.kdl"),
        include_str!("../assets/net/userinfo.kdl"),
    );

    let opts = ParserOpts::new(fs);

    (main_content, PathBuf::from("net/handlers.kdl"), opts)
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
