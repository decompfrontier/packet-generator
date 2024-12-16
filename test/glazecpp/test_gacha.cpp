#include <gtest/gtest.h>
#include "gacha.hpp"

using namespace date;
using namespace std;
//using namespace std::chrono;

using tp = std::chrono::system_clock::time_point;

TEST(packetgen, gacha)
{
	constexpr auto p = 2015_y/March/22;

	GachaMstData d1;
	d1.bg_img = "ayo test";
	d1.start_date = tp{ sys_days{p} };

	GachaMst d;
	d.data.emplace_back(d1);

	std::string buffer = "";
	auto ec = glz::write_json(d, buffer);

	ASSERT_EQ(ec.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"a3vSYuq2\":{\"Kn51uR4Y\":\"yooooooo\"},\"F4q6i9xe\":{\"aV6cLn3v\":\"\",\"Hhgi79M1\":\"REQUEST\"}}");

}
