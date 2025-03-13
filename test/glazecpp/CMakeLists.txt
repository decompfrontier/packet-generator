cmake_minimum_required(VERSION 3.24)
project(glazecpp_test)

include(GoogleTest)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)

FetchContent_Declare(
  glaze
  GIT_REPOSITORY https://github.com/stephenberry/glaze.git
  GIT_TAG main
  GIT_SHALLOW TRUE
)

FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)

if (MSVC)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /MP /utf-8 /W3 /permissive-")
else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -fno-permissive")
endif()

# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(googletest)
FetchContent_MakeAvailable(glaze)

add_executable(${PROJECT_NAME} 
  test.cpp
  test_logincampaign.cpp
  test_gme.cpp
  test_gacha.cpp
)
target_include_directories(${PROJECT_NAME} PRIVATE ../../runtime/cpp ${CMAKE_CURRENT_LIST_DIR}/generated)
target_link_libraries(${PROJECT_NAME} PRIVATE GTest::gtest_main glaze::glaze)

gtest_discover_tests(${PROJECT_NAME})
