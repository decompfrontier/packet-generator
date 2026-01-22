#pragma once

/**
        Simple helpers for packet generator types
*/
#include <chrono>
#include <deque>
#include <format>
#include <sstream>

#if __cplusplus <= 201703L
#include "date.h" // now standard in c++20 ^^
#endif

#if __STDCPP_FLOAT32_T__ > 0 ||                                                \
    __STDCPP_FLOAT64_T__ > 0 // for compatibility with Clang for Android NDK
#include <stdfloat>
#endif

namespace pkg {

#if __STDCPP_FLOAT32_T__ > 0
using float32 = std::float32_t;
#else
using float32 = float;
#endif

#if __STDCPP_FLOAT64_T__ > 0
using float64 = std::float64_t;
#else
using float64 = long double;
#endif

using chrono_time = std::chrono::time_point<std::chrono::system_clock,
                                            std::chrono::milliseconds>;

static constexpr bool string_to_chrono(const std::string &in,
                                       chrono_time &out) {
  std::istringstream iss(in);

#if __cplusplus <= 201703L
  iss >> date::parse("%F %T", out);
#else
  iss >> std::chrono::parse("%F %T", out);
#endif

  return !iss.fail();
}

static constexpr bool chrono_to_string(const chrono_time &in,
                                       std::string &out) {
  try {
#if __cplusplus <= 201703L
    const auto floor = date::floor<std::chrono::seconds>(in);
    out = date::format("%F %T", floor);
#else
    const auto floor = std::chrono::floor<std::chrono::seconds>(in);
    out = std::format("{:%F %T}", floor);
#endif
    return true;
  } catch (...) {
    return false;
  }
}

static constexpr bool unix_to_chrono(uint64_t in, chrono_time &out) {
  try {
#if __cplusplus <= 201703L
    const auto &h = date::sys_seconds{std::chrono::seconds(in)};
#else
    const auto &h = std::chrono::sys_seconds{std::chrono::seconds(in)};
#endif
    out = chrono_time(h);
    return true;
  } catch (...) {
    return false;
  }
}

static constexpr bool chrono_to_unix(const chrono_time &in, uint64_t &out) {
  try {
    out =
        std::chrono::duration_cast<std::chrono::seconds>(in.time_since_epoch())
            .count();
    return true;
  } catch (...) {
    return false;
  }
}

template <typename T> using string_list = std::deque<T>;

template <typename T>
constexpr auto str_to_int(std::string_view v, T &result, int base = 10) {
  auto [ptr, ec] = std::from_chars(v.data(), v.data() + v.size(), result, base);
  return ec == std::errc() && ptr == v.data() + v.size();
}

template <typename T>
constexpr auto int_to_str(T input, std::string &result, int base = 10) {
  constexpr size_t buffer_size = 30;
  char buffer[buffer_size] = {};
  auto [ptr, ec] = std::to_chars(buffer, buffer + buffer_size, input, base);
  if (ec == std::errc()) {
    result += buffer;
    return true;
  }

  return false;
}

template <typename T>
static constexpr bool string_list_push_any(std::string_view sv,
                                           string_list<T> &sl) {
  if constexpr (std::is_arithmetic_v<T>) {
    T result = 0;
    if (!str_to_int<T>(sv, result)) {
      return false;
    }

    sl.emplace_back(result);
    return true;
  } else {
    T result = T(sv);
    sl.emplace_back(result);
    return true;
  }
}

template <typename T>
static constexpr bool string_list_pop_any(const T &sl, std::string &out) {
  if constexpr (std::is_arithmetic_v<T>) {
    return int_to_str<T>(sl, out);
  } else {
    out += sl;
    return true;
  }
}

template <typename T>
static constexpr bool string_list_from(string_list<T> &sl, std::string_view sv,
                                       char character) {
  size_t pos = 0, last = 0;

  if (sv.empty())
    return true;

  sl.clear();

  while ((pos = sv.find(character, last)) != std::string::npos) {
    const auto sv_ptr = sv.substr(last, pos - last);
    last = pos + 1;

    if (!string_list_push_any<T>(sv_ptr, sl)) {
      return false;
    }
  }

  const auto sv_ptr = sv.substr(last);
  if (!sv_ptr.empty()) {
    return string_list_push_any<T>(sv_ptr, sl);
  }

  return true;
}

template <typename T>
static constexpr bool string_list_to(const string_list<T> &sl, std::string &out,
                                     char character) {
  out.clear();

  if (sl.empty())
    return true;

  for (const auto &it : sl) {
    if (!string_list_pop_any<T>(it, out)) {
      return false;
    }

    out += character;
  }
  out = out.substr(0, out.size() - 1);
  return true;
}

} // namespace pkg
