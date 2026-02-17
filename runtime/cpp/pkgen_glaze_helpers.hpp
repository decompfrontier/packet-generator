#pragma once

#include "pkgen_helpers.hpp"
#include <glaze/glaze.hpp>

namespace pkg::glaze {

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
    [](ContainerType& container_object, std::string_view in) {    
    
    auto& data = glz::get_member(container_object, MemPtr);

    if (in.empty()) return;
    if (!pkg::string_list_from(data, in, delimiter)) {
        // TODO(arves): Handle error...
    }

    },
    [](const ContainerType& container_object) -> std::string {

    const auto& data = glz::get_member(container_object, MemPtr);
    if (data.empty()) return "";
    std::string output;
    if (!pkg::string_list_to(data, output, delimiter)) {
        // TODO(arves): Handle error...
    }

    return output;
    }
>;

} // namespace pkg::glaze

namespace glz {

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
    pkg::chrono_to_string(value.val, value.as_str);
    serialize<JSON>::op<Opts>(value.as_str, ctx, b, ix);
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
    pkg::chrono_to_unix(value.val, value.timestamp);
    serialize<JSON>::op<Opts>(value.timestamp, ctx, b, ix);
  }
};

} // namespace glz
