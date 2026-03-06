include(GoogleTest)
include(FetchContent)

option(ENABLE_GLAZE_TEST "Enable test with JSON glaze" ON)

if (MSVC)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /MP /utf-8 /W3 /permissive-")
else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -fno-permissive")
endif()

FetchContent_Declare(
  googletest
  GIT_REPOSITORY https://github.com/google/googletest/
  GIT_TAG main
  GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(googletest)

# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)

add_executable(pkgen_test_pkg
	test_pkg.cpp
)
target_link_libraries(pkgen_test_pkg PRIVATE GTest::gtest_main pkgen_cpp)
gtest_discover_tests(pkgen_test_pkg)

if (ENABLE_GLAZE_TEST)
	FetchContent_Declare(
	  glaze
	  GIT_REPOSITORY https://github.com/stephenberry/glaze.git
	  GIT_TAG main
	  GIT_SHALLOW TRUE
	)
	FetchContent_MakeAvailable(glaze)


	add_executable(pkgen_test_glaze
		test_glaze_unsorted.cpp
		test_glaze_array.cpp
		test_glaze_single_array.cpp
	)
	target_link_libraries(pkgen_test_glaze PRIVATE GTest::gtest_main glaze::glaze pkgen_cpp)
	gtest_discover_tests(pkgen_test_glaze)
endif()
