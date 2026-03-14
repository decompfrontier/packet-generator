#include <pkgen_glaze_helpers.hpp>
#include <gtest/gtest.h>
//#include "perf_test.hpp"

using namespace std::chrono;

#if 0
struct OrderTest1
{
	int a;
	int b;
	std::string c;
};

struct Kontainer1
{
	std::vector<OrderTest1> cuao;
};

struct OrderTest2
{
	int a;
	std::string c;
	int b;
};

struct Kontainer2
{
	std::vector<OrderTest2> cuao;
};

using std::chrono::high_resolution_clock;
using std::chrono::milliseconds;


TEST(glaze, perf_1)
{
	auto t1 = high_resolution_clock::now();
	auto ec = glz::read_json<Kontainer1>((const char*)_acperf_test);
	auto t2 = high_resolution_clock::now();
	ASSERT_TRUE(ec.has_value());

	/*std::string s1 = std::format("{:%Y%m%d%H%M}", t1);
	RecordProperty("ExecutionStart", s1);
	std::string s2 = std::format("{:%Y%m%d%H%M}", t2);
	RecordProperty("ExecutionEnd", s2);*/
	auto ms_int = duration_cast<milliseconds>(t2 - t1).count();
	RecordProperty("ExecutionDiff", ms_int);
}


TEST(glaze, perf_2)
{
	auto t1 = high_resolution_clock::now();
	auto ec = glz::read_json<Kontainer2>((const char*)_acperf_test);
	auto t2 = high_resolution_clock::now();
	ASSERT_TRUE(ec.has_value());

	/*std::string s1 = std::format("{:%Y%m%d%H%M}", t1);
	RecordProperty("ExecutionStart", s1);
	std::string s2 = std::format("{:%Y%m%d%H%M}", t2);
	RecordProperty("ExecutionEnd", s2);*/
	auto ms_int = duration_cast<milliseconds>(t2 - t1).count();
	RecordProperty("ExecutionDiff", ms_int);
}
#endif

struct TimeContainer {
	pkg::chrono_time a;
	pkg::chrono_time b;
};

template <>
struct glz::meta<TimeContainer> {
	using T = TimeContainer;
	static constexpr auto value = object(
		"a", pkg::glaze::datetime<&T::a>(),
		"b", pkg::glaze::datetime_unix<&T::b>()
		);
};

TEST(glaze, datetime_json) {

	constexpr auto FIXED_DATE_1 = 2018y/September/12;
	const auto date_1 = pkg::chrono_time(sys_days(FIXED_DATE_1));
	constexpr auto FIXED_DATE_2 = 2026y/January/20;
	const auto date_2 = pkg::chrono_time(sys_days(FIXED_DATE_2) + 23h + 12min + 01s);
	const auto input = R"({"a":"2018-09-12 00:00:00","b":1768950721})"; // GMT: Tuesday, January 20, 2026 11:12:01 PM
	const auto ec = glz::read_json<TimeContainer>(input);
	ASSERT_TRUE(ec.has_value());
	const auto [a, b] = ec.value();
	ASSERT_EQ(a, date_1);
	ASSERT_EQ(b, date_2);
	std::string buffer;
	TimeContainer tc = ec.value();
	const auto ec2 = glz::write_json(tc, buffer);
	ASSERT_FALSE(ec2);
	ASSERT_EQ(buffer, input);
}

struct SimpleA {
	bool a;
};

template <>
struct glz::meta<SimpleA> {
	using T = SimpleA;
	static constexpr auto value = object("a", pkg::glaze::bool_as_string<&T::a>());
};

TEST(glaze, bool_as_str) {
	const auto input = R"({"a":"1"})";
	const auto ec = glz::read_json<SimpleA>(input);
	ASSERT_TRUE(ec.has_value());
	ASSERT_TRUE(ec.value().a);
	std::string buffer;
	SimpleA tc = ec.value();
	const auto ec2 = glz::write_json(tc, buffer);
	ASSERT_FALSE(ec2);
	ASSERT_EQ(buffer, input);
	const auto bad_input = R"({"a":"655"})";
	const auto ec3 = glz::read_json<SimpleA>(bad_input);
	ASSERT_EQ(ec3.error(), glz::error_code::expected_true_or_false);
}
