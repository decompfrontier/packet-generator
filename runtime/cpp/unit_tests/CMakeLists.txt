include(GoogleTest)
include(FetchContent)

FetchContent_Declare(
  glaze
  GIT_REPOSITORY https://github.com/stephenberry/glaze.git
  GIT_TAG main
  GIT_SHALLOW TRUE
)

FetchContent_Declare(
  googletest
  GIT_REPOSITORY https://github.com/google/googletest/
  GIT_TAG main
  GIT_SHALLOW TRUE
)

#macro(generate_glaze SCHEMA_NAME)
#  set(MYCMD python packet-generator.py -l c++ -s ${SCHEMA_NAME} -o ${CMAKE_CURRENT_BINARY_DIR}/generated)
#  message(STATUS "Executing \"${MYCMD}\"...")
#  execute_process(COMMAND ${MYCMD}
#  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/../../
#  OUTPUT_VARIABLE out
#  ERROR_VARIABLE err
#  RESULT_VARIABLE res)
#
#  message(STATUS "     Result: ${out} (${res})")
#
#  if (NOT "${res}" STREQUAL "0")
#    message(FATAL_ERROR "Execution failed (${res}):\n${out}\n${err}")
#  endif()
#
#endmacro()

if (MSVC)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /MP /utf-8 /W3 /permissive-")
else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -fno-permissive")
endif()

# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(googletest)
FetchContent_MakeAvailable(glaze)

#generate_glaze(net.signalkey)
#generate_glaze(net.challenge_arena)
#generate_glaze(net.daily_login)
#generate_glaze(net.logincampaign)
#generate_glaze(net.gme)
#generate_glaze(mst.npc)
#generate_glaze(mst.gatcha)
#generate_glaze(mst.town)
#generate_glaze(mst.banner)

add_executable(pkgen_tests
#  test.cpp
#  test_logincampaign.cpp
#  test_gme.cpp
#  test_gacha.cpp
#  test_list.cpp
        test_pkg.cpp
        test_glaze_unsorted.cpp
)

target_include_directories(pkgen_tests PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/generated)
target_link_libraries(pkgen_tests PRIVATE GTest::gtest_main glaze::glaze pkgen_cpp)

gtest_discover_tests(pkgen_tests)
