#pragma once

/**
        Simple helpers for pkggen types
*/
#include <chrono>
#include <cmath>
#include <sstream>
#include <unordered_map>

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

template <typename T> using inner_type_v = T::value_type;
template <typename T>
constexpr bool is_arithmetic_inner_v = std::is_arithmetic_v<inner_type_v<T>>;

} // namespace pkg
