cmake_minimum_required(VERSION 3.31)
project(pkgen_cpp)

set(CMAKE_CXX_STANDARD 23) # glaze requires C++23 or greater
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(PKGEN_ENABLE_UNIT_TESTS "Enables Packet-generator unit tests" OFF)

add_library(${PROJECT_NAME} INTERFACE
    date.h
    pkgen_glaze_helpers.hpp
    pkgen_helpers.hpp
)

target_include_directories(${PROJECT_NAME} INTERFACE ${CMAKE_CURRENT_LIST_DIR})

if (MSVC)
    target_compile_options(${PROJECT_NAME} INTERFACE "/Zc:__cplusplus")
endif()

# We do not add a direct dependency of glaze because
#  an implementor could use different runtimes
#  therefore we leave that implementation defined.


if (PKGEN_ENABLE_UNIT_TESTS)
    add_subdirectory(unit_tests)
endif()
