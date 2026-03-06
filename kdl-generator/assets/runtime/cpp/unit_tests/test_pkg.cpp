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

TEST(pkg, parse_string_list_at)
{
  pkg::string_list<uint64_t> p;
  const auto r = pkg::string_list_from(p, "56@24@65@13", '@');
  ASSERT_TRUE(r);
  ASSERT_EQ(p.size(), 4);
  ASSERT_EQ(p[0], 56);
  ASSERT_EQ(p[1], 24);
  ASSERT_EQ(p[2], 65);
  ASSERT_EQ(p[3], 13);
}

TEST(pkg, parse_string_list_comma)
{
  pkg::string_list<uint64_t> p;
  const auto r = pkg::string_list_from(p, "56,24,65,13", ',');
  ASSERT_TRUE(r);
  ASSERT_EQ(p.size(), 4);
  ASSERT_EQ(p[0], 56);
  ASSERT_EQ(p[1], 24);
  ASSERT_EQ(p[2], 65);
  ASSERT_EQ(p[3], 13);
}

TEST(pkg, parse_string_list_error)
{
  pkg::string_list<uint64_t> p;
  const auto r = pkg::string_list_from(p, "56,24,65,123RERRERR", ',');
  ASSERT_FALSE(r);
}

TEST(pkg, unparse_string_list_at)
{
  pkg::string_list<uint64_t> p;
  p.emplace_back(354);
  p.emplace_back(65);
  p.emplace_back(13);
  std::string o;
  const auto r = pkg::string_list_to(p, o, '|');
  ASSERT_TRUE(r);
  ASSERT_EQ(o, "354|65|13");
}

TEST(pkg, unparse_string_list_data)
{
  pkg::string_list<std::string> p;
  p.emplace_back("356");
  p.emplace_back("alepflp");
  p.emplace_back("@@@@@");
  std::string o;
  const auto r = pkg::string_list_to(p, o, '|');
  ASSERT_TRUE(r);
  ASSERT_EQ(o, "356|alepflp|@@@@@");
}

TEST(pkg, parse_string_list_data)
{
  pkg::string_list<std::string> p;
  const auto r = pkg::string_list_from(p, "56,24,65,123RERRERR", ',');
  ASSERT_TRUE(r);
  ASSERT_EQ(p.size(), 4);
  ASSERT_EQ(p[0], "56");
  ASSERT_EQ(p[1], "24");
  ASSERT_EQ(p[2], "65");
  ASSERT_EQ(p[3], "123RERRERR");
}

TEST(pkg, parse_string_list_unk)
{
  pkg::string_list<std::string> p;
  const auto r = pkg::string_list_from(p, "56,24,65,123RERRERR,", ',');
  ASSERT_TRUE(r);
  ASSERT_EQ(p.size(), 4);
  ASSERT_EQ(p[0], "56");
  ASSERT_EQ(p[1], "24");
  ASSERT_EQ(p[2], "65");
  ASSERT_EQ(p[3], "123RERRERR");
}

using namespace std::chrono;

TEST(pkg, unparse_string_test_complex)
{
  pkg::string_list<pkg::chrono_time> p;
  auto time = pkg::chrono_time{ sys_days{ 2026y / February / 17 } };
  p.emplace_back(time);
  auto time2 = pkg::chrono_time{ sys_days{ 2025y / February / 17 } };
  p.emplace_back(time2);

  std::string o;
  const auto r = pkg::string_list_to(p, o, '|');
  ASSERT_TRUE(r);
  ASSERT_EQ(o, "2026-02-17 00:00:00|2025-02-17 00:00:00");
}
