// MSVC fucking sucks bro (https://developercommunity.visualstudio.com/t/C1605-cpp20-module-in-1712-Preview-3/10770119)
#if defined(__clang__) || defined(__GNUC__)
#include <gtest/gtest.h>

#include "logincampaign.hpp"

TEST(packetgen, logincampaign)
{
    UserLoginCampaignInfo d;
    UserLoginCampaignInfoData d1;
    d1.current_day = 1;
    d1.first_for_the_day = true;
    d1.id = 1;
    d1.total_days = 2;
    d.data.emplace_back(d1);
    d1.current_day = 2;
    d1.first_for_the_day = false;
    d1.id = 2;
    d.data.emplace_back(d1);

    std::string buffer = "";
    auto ec = glz::write_json(d, buffer);

    ASSERT_EQ(ec.ec, glz::error_code::none);
    ASSERT_EQ(buffer, "{\"3da6bd0a\":[{\"ad6i23pO\":1,\"4tswNoV9\":1,\"H1Dkq93v\":1,\"1adb38d5\":2},{\"ad6i23pO\":2,\"4tswNoV9\":0,\"H1Dkq93v\":2,\"1adb38d5\":2}]}");
}

#endif
