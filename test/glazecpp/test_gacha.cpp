#include <gtest/gtest.h>
#include "gacha.hpp"

using namespace date;
using namespace std;

using tp = std::chrono::system_clock::time_point;

TEST(packetgen, gacha)
{
	constexpr auto p = 2015_y/March/22;

	GachaMstData d1;
	d1.bg_img = "ayo test";
	d1.start_date = tp{ sys_days{p} + chrono::hours{3} + chrono::minutes{42} + chrono::seconds{14} };
	d1.once_day_flag = true;

	GachaMst d;
	d.data.emplace_back(d1);

	d1.bg_img = "blabla test2";
	d1.once_day_flag = false;
	d.data.emplace_back(d1);

	std::string buffer;
	const auto& ec = glz::write_json(d, buffer);

	ASSERT_EQ(ec.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"5Y4GJeo3\":[{\"1Dg0vUX3\":\"ayo test\",\"W9ABuJj2\":\"\",\"3sdHQb69\":\"\",\"gVSj32QH\":\"\",\"qp37xTDh\":\"\",\"W2c9g0Je\":\"\",\"uKYf13AH\":\"\",\"SzV0Nps7\":\"1970-01-01 00:00:00\",\"v9TR3cDz\":\"\",\"TCnm1F4v\":0,\"7Ffmi96v\":0,\"4N27mkt1\":\"\",\"J3stQ7jd\":0,\"03UGMHxF\":0,\"4tswNoV9\":\"1\",\"yu18xScw\":0,\"qA7M9EjP\":\"2015-03-22 03:42:14\",\"2HY3jpgu\":\"\",\"S1oz60Hc\":0},{\"1Dg0vUX3\":\"blabla test2\",\"W9ABuJj2\":\"\",\"3sdHQb69\":\"\",\"gVSj32QH\":\"\",\"qp37xTDh\":\"\",\"W2c9g0Je\":\"\",\"uKYf13AH\":\"\",\"SzV0Nps7\":\"1970-01-01 00:00:00\",\"v9TR3cDz\":\"\",\"TCnm1F4v\":0,\"7Ffmi96v\":0,\"4N27mkt1\":\"\",\"J3stQ7jd\":0,\"03UGMHxF\":0,\"4tswNoV9\":\"0\",\"yu18xScw\":0,\"qA7M9EjP\":\"2015-03-22 03:42:14\",\"2HY3jpgu\":\"\",\"S1oz60Hc\":0}]}");
}

TEST(packetgen, gachacat)
{
	GachaCategoryData d1;
	constexpr auto p = 2015_y/March/22;

	d1.id = 45;
	d1.img = "ciaociao";
	d1.start_date = tp{ sys_days{p} };

	GachaCategory d;
	d.data.emplace_back(d1);

	std::string buffer;
	const auto& ec = glz::write_json(d, buffer);

	ASSERT_EQ(ec.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"IBs49NiH\":[{\"2r4EoNt4\":0,\"SzV0Nps7\":0,\"3rCmq58M\":\"\",\"vx9uyQVQ\":45,\"In7lGGLn\":\"ciaociao\",\"qA7M9EjP\":1426982400}]}");
}