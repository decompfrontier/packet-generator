#pragma once

#include <glaze/glaze.hpp>
#include <date.h>

#include <string>
#include <vector>
#include <chrono>

namespace glzhlp
{
	using chronotime = std::chrono::system_clock::time_point;

	template <auto T>
	constexpr auto strbool = glz::custom<
		// out -> structure that holds the data
		// p -> C++ output data
		// in -> string input to read
		[](auto& out, const auto& in)
		{
			// read
			decltype(auto)& p = glz::get_member(out, T);
			p = in == "1";
		},

		// in -> structure that holds the data
		// p -> C++ input to read
		// return -> string output
		[](const auto& in) -> auto
		{
			// write
			decltype(auto) p = glz::get_member(in, T);
			return p ? "1" : "0";
		}
	>;

	template <auto T>
	constexpr auto datetime = glz::custom<
		[](auto& out, const auto& in)
		{
			// read
			decltype(auto)& p = glz::get_member(out, T);
			std::stringstream inss(in);
			chronotime t;
			inss >> date::parse("%F %T", t);
			p = t;
		},
		[](const auto& in) -> auto
		{
			// write
			decltype(auto) p = glz::get_member(in, T);
			return date::format("%F %T", floor<std::chrono::seconds>(p));
		}
	>;

	template <auto T>
	constexpr auto datetimeunix = glz::custom<
		[](auto& out, const auto& in)
		{
			// read
			decltype(auto)& p = glz::get_member(out, T);
			const auto& h = date::sys_seconds{ std::chrono::seconds(in) };
			p = chronotime(h);
		},
		[](const auto& in) -> auto
		{
			// write
			decltype(auto) p = glz::get_member(in, T);
			return std::chrono::duration_cast<std::chrono::seconds>(p.time_since_epoch()).count();
		}
	>;
}
