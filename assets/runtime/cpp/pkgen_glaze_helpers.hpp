#pragma once

#include "pkgen_helpers.hpp"
#include <glaze/glaze.hpp>


namespace pkg::glaze {


#if __STDCPP_FLOAT32_T__ > 0
    #define glz_write_f32(memptr) glz::write_float32<memptr>
#else
    #define glz_write_f32(memptr) memptr
#endif

#if __STDCPP_FLOAT64_T__ > 0 // for compatibility with Clang for Android NDK
    #define glz_write_f64(memptr) glz::write_float64<T>;
#else
    #define glz_write_f64(memptr) memptr
#endif


struct my_ops : glz::opts {
  bool minified = true; // Also skip whitespace parsing for smaller code
  bool error_on_unknown_keys = false; // More lenient parsing
  bool prettify = false;
  bool skip_null_members = false;
};

struct debug_opts : glz::opts {
  bool minified = false;
  bool error_on_unknown_keys = true;
  bool prettify = true;
  bool skip_null_members = false;
};

template <class T> struct single_array_helper {
    T &val; // reference to the data
};

template <auto MemPtr> constexpr decltype(auto) single_array() {
    return [](auto&& val) {
        using T = std::decay_t<decltype(val.*MemPtr)>;
        return single_array_helper<T>{
            const_cast<T&>(val.*MemPtr) }; // NOTE(arves): this is a crappy thing yes...
        };
}

// template <class T, uintmax_t N> struct fixed_array_helper {
//     std::array<T, N>  &val; // reference to the data
// };
//
// template <auto MemPtr> constexpr decltype(auto) fixed_array() {
//     return [](auto&& val) {
//         using T = std::decay_t<decltype(val.*MemPtr)>;
//         return single_array_helper<T>{
//             const_cast<T&>(val.*MemPtr) }; // NOTE(arves): this is a crappy thing yes...
//         };
// }


template <class T> struct bool_as_string_helper {
  std::string as_str;
  T &val; // reference to the boolean
};

template <auto MemPtr> constexpr decltype(auto) bool_as_string() {
  return [](auto &&val) {
    return bool_as_string_helper<std::decay_t<decltype(val.*MemPtr)>>{
        {}, val.*MemPtr};
  };
}

struct datetime_helper {
  std::string as_str{};
  chrono_time &val; // pkg::chrono_time
};

template <auto MemPtr> constexpr decltype(auto) datetime() {
  return [](auto &&val) { return datetime_helper{{}, val.*MemPtr}; };
}

struct datetime_unix_helper {
  chrono_time &val; // pkg::chrono_time
  uint64_t timestamp = 0;
};

template <auto MemPtr> constexpr decltype(auto) datetime_unix() {
  return [](auto &&val) { return datetime_unix_helper{val.*MemPtr, 0}; };
}

template <typename ContainerType, auto MemPtr, char delimiter> requires pkg::detail::containerable<pkg::detail::decltype_real<MemPtr>>
constexpr auto array_string = glz::custom<
    [](ContainerType& container_object, std::string_view in, glz::context& ctx) {

    auto& data = glz::get_member(container_object, MemPtr);

    if (in.empty()) return;
    if (!pkg::string_list_from(data, in, delimiter)) {
        ctx.error = glz::error_code::invalid_variant_string;
    }

    },
    [](const ContainerType& container_object, glz::context& ctx) -> std::string {

    const auto& data = glz::get_member(container_object, MemPtr);
    if (data.empty()) return "";
    std::string output;
    if (!pkg::string_list_to(data, output, delimiter)) {
        ctx.error = glz::error_code::invalid_variant_string;
    }

    return output;
    }
>;


} // namespace pkg::glaze

namespace glz {

template <class T> struct from<JSON, pkg::glaze::single_array_helper<T>> {
    template <auto Opts>
    static void op(auto&& value, auto&& ctx, auto&& it, auto&& end) {
        skip_ws<Opts>(ctx, it, end);
        if (it >= end) {
            ctx.error = error_code::unexpected_end;
            return;
        }
        std::array<T, 1> data;
        parse<JSON>::op<Opts>(data, ctx, it, end);
        value.val = data[0];
    }
};

template <class T> struct to<JSON, pkg::glaze::single_array_helper<T>> {
    template <auto Opts, class B>
    static void op(auto&& value, auto&& ctx, B&& b, auto&& ix) noexcept {
        std::array<T, 1> data { value.val };
        serialize<JSON>::op<Opts>(data, ctx, b, ix);
    }
};

template <class T> struct from<JSON, pkg::glaze::bool_as_string_helper<T>> {
  template <auto Opts>
  static void op(auto &&value, auto &&ctx, auto &&it, auto &&end) {
    skip_ws<Opts>(ctx, it, end);
    if (it >= end) {
      ctx.error = error_code::unexpected_end;
      return;
    }
    parse<JSON>::op<Opts>(value.as_str, ctx, it, end);

    if (value.as_str == "1") {
      value.val = true;
    } else if (value.as_str == "0") {
      value.val = false;
    } else {
      ctx.error = error_code::expected_true_or_false;
    }
  }
};

template <class T> struct to<JSON, pkg::glaze::bool_as_string_helper<T>> {
  template <auto Opts, class B>
  static void op(auto &&value, auto &&ctx, B &&b, auto &&ix) noexcept {
    serialize<JSON>::op<Opts>(value.val ? "1" : "0", ctx, b, ix);
  }
};

template <> struct from<JSON, pkg::glaze::datetime_helper> {
  template <auto Opts>
  static void op(auto &&value, auto &&ctx, auto &&it, auto &&end) {
    skip_ws<Opts>(ctx, it, end);
    if (it >= end) {
      ctx.error = error_code::unexpected_end;
      return;
    }
    parse<JSON>::op<Opts>(value.as_str, ctx, it, end);
    if (!pkg::string_to_chrono(value.as_str, value.val)) {
      ctx.error = error_code::invalid_variant_string;
    }
  }
};

template <> struct to<JSON, pkg::glaze::datetime_helper> {
  template <auto Opts, class B>
  static void op(auto &&value, auto &&ctx, B &&b, auto &&ix) noexcept {
    if (!pkg::chrono_to_string(value.val, value.as_str)) {
        ctx.error = error_code::invalid_variant_string;
    }
    else {
        serialize<JSON>::op<Opts>(value.as_str, ctx, b, ix);
    }
  }
};

template <> struct from<JSON, pkg::glaze::datetime_unix_helper> {
  template <auto Opts>
  static void op(auto &&value, auto &&ctx, auto &&it, auto &&end) {
    skip_ws<Opts>(ctx, it, end);
    if (it >= end) {
      ctx.error = error_code::unexpected_end;
      return;
    }
    parse<JSON>::op<Opts>(value.timestamp, ctx, it, end);
    if (!pkg::unix_to_chrono(value.timestamp, value.val)) {
      ctx.error = error_code::invalid_variant_string;
    }
  }
};

template <> struct to<JSON, pkg::glaze::datetime_unix_helper> {
  template <auto Opts, class B>
  static void op(auto &&value, auto &&ctx, B &&b, auto &&ix) noexcept {
    if (!pkg::chrono_to_unix(value.val, value.timestamp)) {
          ctx.error = error_code::invalid_variant_string;
    }
    else {
        serialize<JSON>::op<Opts>(value.timestamp, ctx, b, ix);
    }
  }
};

} // namespace glz
