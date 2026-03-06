#include <pkgen_glaze_helpers.hpp>
#include <gtest/gtest.h>

struct Ciao {
	std::string a;
};

struct CiaoArray {
	Ciao data;
};

template <>
struct glz::meta<Ciao> {
	using T = Ciao;
	constexpr static auto value = object("_a_", &T::a);
};

template <>
struct glz::meta<CiaoArray> {
	using T = CiaoArray;
	constexpr static auto value = object(
		"ciao", pkg::glaze::single_array<&T::data>()
	);
};

TEST(glaze, single_array) {
	const auto input = R"({"ciao":[{"_a_":"qualcosa"}]})";
	const auto ec = glz::read_json<CiaoArray>(input);
	ASSERT_TRUE(ec.has_value());
	const auto& ecval = ec.value();
	ASSERT_EQ(ecval.data.a, "qualcosa");

	std::string buffer;
	const auto& tc = ec.value();
	const auto ec2 = glz::write_json(tc, buffer);
	ASSERT_FALSE(ec2);
	ASSERT_EQ(buffer, input);

	const auto broken = R"({"ciao":[{"_a_": "uno"},{"_a"_: "due"}]})";
	const auto ec3 = glz::read_json<CiaoArray>(broken);
	ASSERT_FALSE(ec3.has_value());
}
