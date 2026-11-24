cmake_minimum_required(VERSION 3.12)
project(visit_expressions)
# The expected output includes the literals of every type feature. Turn on the option for each
# feature delta_kernel_ffi was built with; the literals of the others are excluded.
option(FLOAT16 "delta_kernel_ffi is built with --features float16." OFF)
option(NANOSECOND_TIMESTAMPS "delta_kernel_ffi is built with --features nanosecond-timestamps." OFF)

add_executable(visit_expression visit_expression.c ../common/kernel_utils.c)
target_compile_definitions(visit_expression PUBLIC DEFINE_DEFAULT_ENGINE_BASE)
target_include_directories(visit_expression PUBLIC 
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../target/ffi-headers"
    "${CMAKE_CURRENT_SOURCE_DIR}/../common")
target_link_directories(visit_expression PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../../../target/debug")
target_link_libraries(visit_expression PUBLIC delta_kernel_ffi)
target_compile_options(visit_expression PUBLIC)

if(MSVC)
  target_compile_options(visit_expression PRIVATE /W4 /WX)
else()
  # no-strict-prototypes because arrow headers have fn defs without prototypes
  target_compile_options(visit_expression PRIVATE -Wall -Wextra -Wpedantic -Werror -Wno-strict-prototypes -g -fsanitize=address)
  target_link_options(visit_expression PRIVATE -g -fsanitize=address)
endif()

# Add the kernel expresion -> engine expression test
include(CTest)
set(ExprTestRunner "../../../tests/test-expression-visitor/run_test.sh")
set(ExprExpectedPath "../../../tests/test-expression-visitor/expected.txt")
set(ExprExcludedLiterals "")
if(NOT FLOAT16)
  list(APPEND ExprExcludedLiterals Float16)
endif()
if(NOT NANOSECOND_TIMESTAMPS)
  list(APPEND ExprExcludedLiterals TimestampNanos TimestampNanosNtz)
endif()
list(JOIN ExprExcludedLiterals "|" ExprExcludedLiterals)
add_test(NAME test_expression_visitor
  COMMAND ${ExprTestRunner} ${ExprExpectedPath} "${ExprExcludedLiterals}")
