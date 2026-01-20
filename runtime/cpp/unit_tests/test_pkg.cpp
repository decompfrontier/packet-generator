#include <pkgen_helpers.hpp>
#include <gtest/gtest.h>

TEST(pkg, parse_date_time) {
  pkg::chrono_time t;
  std::string s = "2020-02-07 15:00:00";
  ASSERT_TRUE(pkg::string_to_chrono(s, t));

  s = "EEEEEEEEEe020-02-07 15:00:00";
  ASSERT_FALSE(pkg::string_to_chrono(s, t));
}

TEST(pkg, unparse_date_time) {
  pkg::chrono_time t;
  std::string s = "2020-02-07 15:00:00";
  ASSERT_TRUE(pkg::string_to_chrono(s, t));
  ASSERT_TRUE(pkg::chrono_to_string(t, s));
  ASSERT_EQ("2020-02-07 15:00:00", s);
}

TEST(pkg, parse_unix_date_time) {
  pkg::chrono_time t;
  uint64_t s = 1768950721;
  ASSERT_TRUE(pkg::unix_to_chrono(s, t));

  uint64_t g = 0;
  ASSERT_TRUE(pkg::chrono_to_unix(t, g));
  ASSERT_EQ(s, g);
}

