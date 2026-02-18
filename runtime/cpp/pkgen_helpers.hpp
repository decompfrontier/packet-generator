#pragma once

/**
        Simple helpers for packet generator types
*/
#include <chrono>
#include <deque>
#include <format>
#include <sstream>
#include <array>
#include <unordered_map>

#if __cplusplus <= 201703L
#include "date.h" // now standard in c++20 ^^
#endif

#if __STDCPP_FLOAT32_T__ > 0 ||                                                \
    __STDCPP_FLOAT64_T__ > 0 // for compatibility with Clang for Android NDK
#include <stdfloat>
#endif

namespace pkg {

    // -- types

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

template <typename T> using string_list = std::deque<T>;

    // -- chrono conversions

static bool string_to_chrono(std::string_view in,
                                       chrono_time &out) {

  std::string a = std::string(in);
  std::istringstream iss(a);

#if __cplusplus <= 201703L
  iss >> date::parse("%F %T", out);
#else
  iss >> std::chrono::parse("%F %T", out);
#endif

  return !iss.fail();
}

static bool chrono_to_string(const chrono_time &in,
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

    // -- unix chrono conversions

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

    // -- string int conversions

template <std::integral T>
constexpr auto str_to_int(std::string_view v, T& result, int base = 10) {
    auto [ptr, ec] = std::from_chars(v.data(), v.data() + v.size(), result, base);
    return ec == std::errc() && ptr == v.data() + v.size();
}

template <std::integral T>
static auto int_to_str(T input, std::string& result, int base = 10) {
    constexpr size_t buffer_size = 30;
    char buffer[buffer_size] = {};
    auto [ptr, ec] = std::to_chars(buffer, buffer + buffer_size, input, base);
    if (ec == std::errc()) {
        result += buffer;
        return true;
    }

    return false;
}

    // -- meta function to support string list converion

namespace detail {

    // -- (source: https://stackoverflow.com/questions/54783958/stripping-the-class-from-a-methodtype-obtained-via-decltype)
    template <class MemberPointer>
    struct as_free_pointer;

    template <class C, class T>
    struct as_free_pointer<T C::*> {
        using type = T*;
    };

    template <class MemberPointer>
    using as_free_pointer_t = typename as_free_pointer<MemberPointer>::type;
    // --

    template <typename T>
    concept value_typable = requires {
        typename T::value_type;
    };

    template <auto T>
    using decltype_real = std::remove_pointer_t<as_free_pointer_t<decltype(T)>>;

    template <class T>
    concept containerable = requires(T a) {
        value_typable<T>;
        { a.clear() };
        { a.empty() } -> std::convertible_to<bool>;
        { a.emplace_back };
    };

    // conversion string -> string

    template <class T>
    concept is_string = std::same_as<std::string, std::decay_t<T>>;

    template <is_string T>
    static bool to(const T& input, std::string& result) {
        result = input;
        return true;
    }

    template <is_string T>
    static bool from(std::string_view sv, T& output) {
        output = sv;
        return true;
    }

    // conversion int -> string

    template <std::integral T>
    static bool to(const T& input, std::string& result) {
        return int_to_str<T>(input, result);
    }

    template <std::integral T>
    static bool from(std::string_view sv, T& output) {
        return str_to_int<T>(sv, output);
    }

    // conversion chrono_time

    template <class T>
    concept is_chrono_time = std::same_as<pkg::chrono_time, std::decay_t<T>>;

    template <is_chrono_time T>
    static bool to(const T& input, std::string& result) {
        return chrono_to_string(input, result);
    }


    template <is_chrono_time T>
    static bool from(std::string_view sv, T& output) {
        return string_to_chrono(sv, output);
    }

    // TODO(arves): Make a concept about a "vector" that depends on value_typable

} // namespace detail

    // -- string list operations

template <detail::containerable T>
static bool string_list_from(T &sl, std::string_view sv,
                                       char character) {
  size_t pos = 0, last = 0;

  if (sv.empty())
    return true;

  sl.clear();

  const auto pusher = [&sl](std::string_view func_sv) -> bool {
      using P = typename T::value_type;
      P data_out = P();
      if (!detail::from<P>(func_sv, data_out)) return false;
      sl.emplace_back(data_out);
      return true;
  };

  while ((pos = sv.find(character, last)) != std::string::npos) {
    const auto sv_ptr = sv.substr(last, pos - last);
    last = pos + 1;

    if (!pusher(sv_ptr)) {
      return false;
    }
  }

  const auto sv_ptr = sv.substr(last);
  if (!sv_ptr.empty()) {
    return pusher(sv_ptr);
  }

  return true;
}

template <detail::containerable T>
static bool string_list_to(const T &sl, std::string &out,
                                     char character) {
  out.clear();

  if (sl.empty())
    return true;

  for (const auto &it : sl) {
    std::string tmp;
    if (!detail::to<typename T::value_type>(it, tmp)) {
      return false;
    }

    out += tmp + character;
  }
  out = out.substr(0, out.size() - 1);
  return true;
}

} // namespace pkg
