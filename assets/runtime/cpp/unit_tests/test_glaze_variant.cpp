#include <pkgen_glaze_helpers.hpp>
#include <gtest/gtest.h>


/// Content of the game request/response.
struct GmeBody {

    /// Encrypted JSON content.
    std::string body;
};


/// Header of a game request/response.
struct GmeHeader {

    /// ID of the request
    std::string id;

    /// ID of the client that invoked the request
    std::string client_id;
};


/// Object that stores any possible error with the request or response.
struct GmeError {

    /// Message to show on the error.
    std::string message;

    /// URL to open in the browser after OK is pressed. (like for update the game)
    std::string url;
};


template <>
struct glz::meta<GmeHeader> {
    using T = GmeHeader;
    static constexpr auto value = object(
        "Hhgi79M1", &T::id,
        "aV6cLn3v", &T::client_id
    );
};

template <>
struct glz::meta<GmeBody> {
    using T = GmeBody;
    static constexpr auto value = object(
        "Kn51uR4Y", &T::body
    );
};

template <>
struct glz::meta<GmeError> {
    using T = GmeError;
    static constexpr auto value = object(
        //"3e9aGpus", &T::flag,
        //"iPD12YCr", &T::cmd,
        "ZC0msu2L", &T::message,
        "zcJeTx18", &T::url
    );
};

struct GmeErrorOuter {
    GmeError error;
};

template <>
struct glz::meta<GmeErrorOuter> {
    static constexpr auto value = object(
        "b5PH6mZa", &GmeErrorOuter::error
    );
};

struct GmeBodyOuter {
    GmeBody body;
};

template <>
struct glz::meta<GmeBodyOuter> {
    static constexpr auto value = object(
        "a3vSYuq2", &GmeBodyOuter::body
    );
};


/// Main packet of interaction between client and server.
struct GmeAction {

    /// Header of the message.
    GmeHeader header;

    /// Body of the message (or error).
    std::variant<GmeBodyOuter, GmeErrorOuter> data;
};


template <>
struct glz::meta<GmeAction> {
    using T = GmeAction;
    static constexpr auto value = object(
        "F4q6i9xe", &T::header,
        "q", &T::data
    );
};


TEST(variant, test1) {
    const auto json1 = R"({"F4q6i9xe":{"Hhgi79M1":"ciao","aV6cLn3v":"aaa"},"a3vSYuq2":{"Kn51uR4Y":"qualcosa"}})";
    const auto ec = glz::read_json<GmeAction>(json1);
    ASSERT_TRUE(ec.has_value());
}
