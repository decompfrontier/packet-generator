#pragma once

#include <glaze/glaze.hpp>
#include <date.h>

#include <string>
#include <vector>
#include <chrono>
#include <stdfloat>

#if __STDCPP_FLOAT32_T__ != 1
#define glzhlp_float32 float
#define glzhlp_write_float32(memptr) memptr
#else
#define glzhlp_float32 std::float32_t
#define glzhlp_write_float32(memptr) glz::write_float32< memptr >
#endif

#if __STDCPP_FLOAT64_T__ != 1
#define glzhlp_float64 long double
#define glzhlp_write_float64(memptr) memptr
#else
#define glzhlp_float64 std::float64_t
#define glzhlp_write_float64(memptr) glz::write_float64< memptr >
#endif

namespace glzhlp
{
	using chronotime = std::chrono::system_clock::time_point;

	template <typename P, auto T>
	constexpr auto strbool = glz::custom<
		// out -> structure that holds the data
		// p -> C++ output data
		// in -> string input to read
		[](P& out, glz::sv in)
		{
			// read
			bool& p = glz::get_member(out, T);
			p = (in == "1");
		},

		// in -> structure that holds the data
		// p -> C++ input to read
		// return -> string output
		[](const P& in) -> glz::sv
		{
			// write
			bool p = glz::get_member(in, T);
			return p ? "1" : "0";
		}
	>;

	template <typename P, auto T>
	constexpr auto datetime = glz::custom<
		[](P& out, glz::sv in)
		{
			// read
			glzhlp::chronotime& p = glz::get_member(out, T);
			const auto& str = std::string(in);
			std::stringstream inss(str);
			chronotime t;
			inss >> date::parse("%F %T", t);
			p = t;
		},
		[](const P& in) -> glz::sv
		{
			// write
			const glzhlp::chronotime& p = glz::get_member(in, T);
			return date::format("%F %T", floor<std::chrono::seconds>(p));
		}
	>;

	template <typename P, auto T>
	constexpr auto datetimeunix = glz::custom<
		[](P& out, uint64_t in)
		{
			// read
			glzhlp::chronotime& p = glz::get_member(out, T);
			const auto& h = date::sys_seconds{ std::chrono::seconds(in) };
			p = chronotime(h);
		},
		[](const P& in) -> uint64_t
		{
			// write
			const glzhlp::chronotime& p = glz::get_member(in, T);
			return std::chrono::duration_cast<std::chrono::seconds>(p.time_since_epoch()).count();
		}
	>;
}
