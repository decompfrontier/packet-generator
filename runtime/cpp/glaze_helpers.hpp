#pragma once

#include <glaze/glaze.hpp>
#include <date.h>

#include <string>
#include <array>
#include <deque>
#include <optional>
#include <chrono>

namespace glzhlp
{
	using chronotime = std::chrono::system_clock::time_point;

	constexpr const auto read_datetime = [](const std::string& s) -> chronotime {
		std::stringstream in(s);
		chronotime t;
		in >> date::parse("%Y-%m-%d %H:%M:%S", t);
		return t;
	};

	constexpr const auto write_datetime = [](chronotime& c) -> std::string {
		return date::format("%Y-%m-%d %H:%M:%S", c);
	};

	constexpr const auto read_datetimeunix = [](uint64_t s) -> chronotime {
		const auto& h = date::sys_seconds{ std::chrono::seconds(s) };
		return chronotime(h);
	};

	constexpr const auto write_datetimeunix = [](chronotime& c) -> uint64_t {
		return c.time_since_epoch().count();
	};
}
