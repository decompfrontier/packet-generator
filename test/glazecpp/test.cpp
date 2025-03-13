#include <gtest/gtest.h>

#include "signalkey.hpp"
#include "challenge_arena.hpp"
#include "daily_login.hpp"

TEST(packetgen, signalkey)
{
    SignalKey q;
    q.key = "GFHJEIGHJEIFGJHASIEG";

    std::string buffer;

    const auto& ec = glz::write_json(q, buffer);

    ASSERT_EQ(ec.ec, glz::error_code::none);
    ASSERT_EQ(buffer, "[{\"Kn51uR4Y\":\"GFHJEIGHJEIFGJHASIEG\"}]");
}

TEST(packetgen, challengearena)
{
    ChallengeArenaUserInfo d;
    d.league_id = 54;
    d.rainbow_coins = 9;
    d.unkstr2 = "AEAEAEAE";

    std::string buffer;
    const auto& ec = glz::write_json(d, buffer);

    ASSERT_EQ(ec.ec, glz::error_code::none);
    ASSERT_EQ(buffer, "[{\"xZeGgDQe\":54,\"KAZmxkgy\":9,\"h7eY3sAK\":\"\",\"Nou5bCmm\":0,\"AKP8t3xK\":0,\"e34YV1Ey\":0,\"4lH05mQr\":0,\"BcIqcWDM\":0,\"fBGCdi8I\":0,\"zf5Ae850\":0,\"outas79f\":\"AEAEAEAE\"}]");
}

TEST(packetgen, dailylogin)
{
    DailyLoginRewardsUserInfo d;
    d.current_day = 4;
    d.id = 1;
    d.message = " day(s) more to guaranteed Gem!";
    d.next_reward_id = 4;
    d.remaining_days_till_guaranteed_reward = 2;
    d.user_current_count = 9;
    d.user_spin_limit_count = 11;

    std::string buffer;
    const auto& ec = glz::write_json(d, buffer);

    ASSERT_EQ(ec.ec, glz::error_code::none);
    ASSERT_EQ(buffer, "{\"ad6i23pO\":4,\"XIvaD6Jp\":1,\"ZC0msu2L\":\" day(s) more to guaranteed Gem!\",\"outas79f\":4,\"u8iD6ka7\":2,\"35JXN4Ay\":9,\"5xStG99s\":11}");
}
