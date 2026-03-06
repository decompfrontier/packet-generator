#include <pkgen_glaze_helpers.hpp>
#include <gtest/gtest.h>

using namespace std::chrono;

struct SimpleArrayElement {
	int a;
};

struct MyData {
	std::deque<SimpleArrayElement> ayo;
};

template <>
struct glz::meta<SimpleArrayElement> {
	using T = SimpleArrayElement;
	constexpr static auto value = object(
		"la_a", &T::a
	);
};

template <>
struct glz::meta<MyData> {
	using T = MyData;
	constexpr static auto value = object(
		"campo", &T::ayo
	);
};

TEST(glaze, normal_array) {
	const auto input = R"({"campo":[{"la_a":45},{"la_a":9999}]})";
	const auto ec = glz::read_json<MyData>(input);
	ASSERT_TRUE(ec.has_value());
	const auto& ecval = ec.value();
	ASSERT_EQ(ecval.ayo.size(), 2);
	ASSERT_EQ(ecval.ayo[0].a, 45);
	ASSERT_EQ(ecval.ayo[1].a, 9999);

	std::string buffer;
	const auto& tc = ec.value();
	const auto ec2 = glz::write_json(tc, buffer);
	ASSERT_FALSE(ec2);
	ASSERT_EQ(buffer, input);
}

struct MyData2 {
	pkg::string_list<pkg::chrono_time> ayo;
};

template <>
struct glz::meta<MyData2> {
	using T = MyData2;
	constexpr static auto value = object(
		"campo", pkg::glaze::array_string<T, &T::ayo, '@'>
	);
};

TEST(glaze, string_array) {
	const auto time = pkg::chrono_time{ sys_days{ 2026y / February / 17 } };
	const auto time2 = pkg::chrono_time{ sys_days{ 2025y / February / 17 } };
	const auto input = R"({"campo":"2026-02-17 00:00:00@2025-02-17 00:00:00"})";
	const auto ec = glz::read_json<MyData2>(input);
	ASSERT_TRUE(ec.has_value());
	const auto& ecval = ec.value();
	ASSERT_EQ(ecval.ayo.size(), 2);
	ASSERT_EQ(ecval.ayo[0], time);
	ASSERT_EQ(ecval.ayo[1], time2);

	std::string buffer;
	const auto& tc = ec.value();
	const auto ec2 = glz::write_json(tc, buffer);
	ASSERT_FALSE(ec2);
	ASSERT_EQ(buffer, input);
}
