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

	template <class T, typename = void>
	struct inner_type {
		using value = T;
	};

	template <class T>
	struct inner_type<T, std::void_t<typename T::value_type>> : inner_type<typename T::value_type> {};

	template <class T>
	using inner_type_v = typename inner_type<T>::value;

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
		[](const P& in) -> std::string
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

	template <typename T> requires std::is_same_v<inner_type_v<T>, uint32_t>
	constexpr auto str_to_int(const std::string_view v, int base = 10)
	{
		return std::stoul(v, nullptr, base);
	}

	template <typename T> requires std::is_same_v<inner_type_v<T>, uint64_t>
	constexpr auto str_to_int(const std::string_view v, int base = 10)
	{
		return std::stoull(v, nullptr, base);
	}

	template <typename P, auto T>
	constexpr auto commalist = glz::custom<
		[](P& out, glz::sv in)
		{
			// read
			auto& p = glz::get_member(out, T);
			p.clear();

			size_t pos = 0, lastpos = 0;
			while ((pos = in.find(',', lastpos)) != std::string::npos)
			{
				const auto cur = in.substr(lastpos, pos - lastpos);
				lastpos = pos + 1;

				if constexpr (std::is_arithmetic_v<decltype(p)>)
				{
					p.emplace_back(str_to_int<decltype(p)>(cur));
				}
				else
				{
					p.emplace_back(cur);
				}
			}

			const auto cur = in.substr(lastpos);
			if constexpr (std::is_arithmetic_v<decltype(p)>)
			{
				p.emplace_back(str_to_int<decltype(p)>(cur));
			}
			else
			{
				p.emplace_back(cur);
			}

		},
		[](const P& in) -> std::string
		{
			// write
			const auto& data = glz::get_member(in, T);
			std::string so;
			for (const auto& v : data)
			{
				so += ",";
				if constexpr (std::is_arithmetic_v<decltype(data)>)
				{
					so += std::to_string(v);
				}
				else
				{
					so += v;
				}
			}
			so = so.substr(1);
			return so;
		}
	>;
}
