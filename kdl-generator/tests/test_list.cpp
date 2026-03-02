#include <gtest/gtest.h>
#include <mst_banner.hpp>
#include <mst_town.hpp>

TEST(packetgen, bannerlist)
{
	BannerInfoMst mst;
	BannerInfoMstData a;
	a.display_order = 1;
	a.id = 1;
	a.image = 0;
	a.name = "CIAO";
	a.page_type = "ee";
	a.param = "param";
	a.read_count = 34;
	a.url = "http://q.q.q.q";
	a.target_os.emplace_back(v_BannerOperativeSystem::Android);
	a.target_os.emplace_back("9");
	a.target_os.emplace_back("4");
	mst.data.emplace_back(a);

	std::string buffer;
	const auto& ec = glz::write_json(mst, buffer);

	ASSERT_EQ(ec.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"Pk5F1vhx\":[{\"XuJL4pc5\":\"1\",\"oL71Fz3a\":\"1\",\"1gDkL6XR\":0,\"NyYKc1A5\":\"CIAO\",\"LM34kfVC\":\"ee\",\"t5R47iwj\":\"param\",\"d36D1g8T\":\"34\",\"aL70hVYQ\":\"2,9,4\",\"jsRoN50z\":\"http://q.q.q.q\"}]}");

	BannerInfoMst mst2;
	const auto& ec2 = glz::read_json(mst2, buffer);
	ASSERT_EQ(ec2.ec, glz::error_code::none);

	ASSERT_EQ(mst2.data.size(), 1);
	const auto& b = mst2.data[0];
	ASSERT_EQ(b.display_order, 1);
	ASSERT_EQ(b.id, 1);
	ASSERT_EQ(b.image, 0);
	ASSERT_EQ(b.name, "CIAO");
	ASSERT_EQ(b.page_type, "ee");
	ASSERT_EQ(b.param, "param");
	ASSERT_EQ(b.read_count, 34);
	ASSERT_EQ(b.url, "http://q.q.q.q");
	ASSERT_EQ(b.target_os.size(), 3);
	ASSERT_EQ(b.target_os[0], v_BannerOperativeSystem::Android);
	ASSERT_EQ(b.target_os[1], "9");
	ASSERT_EQ(b.target_os[2], "4");
}

TEST(packetgen, townlist)
{
	TownFacilityLvMst mst;
	TownFacilityLvMstData d;
	d.id = 4;
	d.karma = 34;
	d.lv = 9;
	d.release_receipe.emplace_back(100);
	d.release_receipe.emplace_back(75);
	d.release_receipe.emplace_back(88);
	d.release_receipe.emplace_back(33);
	mst.data.emplace_back(d);

	std::string buffer;
	const auto& ec = glz::write_json(mst, buffer);
	ASSERT_EQ(ec.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"d0EkJ4TB\":[{\"y9ET7Aub\":4,\"HTVh8a65\":34,\"D9wXQI2V\":9,\"rGoJ6Ty9\":\"100,75,88,33\"}]}");
}
