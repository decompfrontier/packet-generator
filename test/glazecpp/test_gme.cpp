#include <gtest/gtest.h>
#include "gme.hpp"

TEST(packetgen, gme)
{
	GmeAction act;
	std::string buffer;

	act.header.id = "REQUEST";

	act.body = GmeBody();
	act.body.value().body = "yooooooo";

	const auto& ec = glz::write_json(act, buffer);

	ASSERT_EQ(ec.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"a3vSYuq2\":{\"Kn51uR4Y\":\"yooooooo\"},\"F4q6i9xe\":{\"aV6cLn3v\":\"\",\"Hhgi79M1\":\"REQUEST\"}}");

	act.body.reset();

	act.error = GmeError();
	act.error.value().message = "OPZ";

	const auto& ec2 = glz::write_json(act, buffer);

	ASSERT_EQ(ec2.ec, glz::error_code::none);
	ASSERT_EQ(buffer, "{\"b5PH6mZa\":{\"iPD12YCr\":0,\"3e9aGpus\":0,\"ZC0msu2L\":\"OPZ\",\"zcJeTx18\":\"\"},\"F4q6i9xe\":{\"aV6cLn3v\":\"\",\"Hhgi79M1\":\"REQUEST\"}}");
}
