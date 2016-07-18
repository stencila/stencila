cmake_minimum_required(VERSION 2.6)
project(Stencila)

# Define lists of source and test files
file(GLOB_RECURSE SOURCES stencila/*.cpp)
file(GLOB_RECURSE TESTS tests/*.cpp)

# Define include directories for all targets
# in this file
include_directories(
	.
	${CMAKE_BINARY_DIR}
	${CMAKE_BINARY_DIR}/requires/boost
	${CMAKE_BINARY_DIR}/requires/cmark/src ${CMAKE_BINARY_DIR}/requires/cmark/build/src
#	${CMAKE_BINARY_DIR}/requires/cpr/include
	${CMAKE_BINARY_DIR}/requires/jsoncpp/dist
	${CMAKE_BINARY_DIR}/requires/libgit2/include
	${CMAKE_BINARY_DIR}/requires/libzip/lib ${CMAKE_BINARY_DIR}/requires/libzip/build
	${CMAKE_BINARY_DIR}/requires/pugixml/src
	${CMAKE_BINARY_DIR}/requires/tidy-html5/include
	${CMAKE_BINARY_DIR}/requires/websocketpp
)

# Define compiler options for all targets
# in this file (may be augmented below)
add_compile_options(
	-std=c++11 -O2
	-Wall -Wno-unused-variable
)
if("${CMAKE_CXX_COMPILER_ID}" MATCHES "Clang")
	add_compile_options(
		-Wno-unknown-pragmas # prevents many warnings
		-Wno-missing-braces # prevents unecessary warnings, see http://stackoverflow.com/a/13905432/4625911
	)
endif()
if(WIN32)
	add_compile_options(
		-Wno-strict-aliasing # prevents warnings in boost/thread/win32/shared_mutex.hpp
	)
else()
	add_compile_options(
		-fPIC # because eventually linked into shared dynamic libraries
	)	
endif()


# Create version.cpp (only if it has changed to prevent unecessary builds)
execute_process(COMMAND ../../version.sh OUTPUT_VARIABLE VERSION OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process(COMMAND git rev-parse HEAD OUTPUT_VARIABLE COMMIT OUTPUT_STRIP_TRAILING_WHITESPACE)
file(WRITE ${CMAKE_BINARY_DIR}/version.cpp.txt
	"#include <stencila/version.hpp>\n"
	"const std::string Stencila::version = \"${VERSION}\";\n"
	"const std::string Stencila::commit = \"${COMMIT}\";\n"
)
execute_process(
	COMMAND ${CMAKE_COMMAND} -E copy_if_different version.cpp.txt version.cpp
)

# Generate syntax lexer using Flex and parser using Lemon
function(generate_syntax lang)
	# Lexer
	add_custom_command(
		OUTPUT ${CMAKE_BINARY_DIR}/syntax-${lang}-lexer.hpp ${CMAKE_BINARY_DIR}/syntax-${lang}-lexer.cpp
		COMMAND flex --outfile syntax-${lang}-lexer.cpp --header-file=syntax-${lang}-lexer.hpp ../stencila/syntax-${lang}.l
		WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
		DEPENDS stencila/syntax-${lang}.l
	)
	set_source_files_properties(
		${CMAKE_BINARY_DIR}/syntax-${lang}-lexer.cpp PROPERTIES COMPILE_FLAGS "-Wno-unused-function -Wno-sign-compare"
	)
	# Parser
	add_custom_command(
		OUTPUT ${CMAKE_BINARY_DIR}/syntax-${lang}-parser.h ${CMAKE_BINARY_DIR}/syntax-${lang}-parser.cpp
		COMMAND cp ../stencila/syntax-${lang}.y .
		COMMAND lemon syntax-${lang}.y
		COMMAND mv syntax-${lang}.c syntax-${lang}-parser.cpp
		WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
		DEPENDS stencila/syntax-${lang}.y
	)
	set_source_files_properties(
		${CMAKE_BINARY_DIR}/syntax-${lang}-parser.cpp PROPERTIES COMPILE_FLAGS "-Wno-sign-compare"
	)
	# Jump through hoops to get title case language
	string(SUBSTRING ${lang} 0 1 lang_first)
	string(TOUPPER ${lang_first} lang_first)
	string(REGEX REPLACE "^.(.*)" "${lang_first}\\1" lang_title "${lang}")
	# Generate parse function from template file to use lexer and parser
	add_custom_command(
		OUTPUT ${CMAKE_BINARY_DIR}/syntax-${lang}-parse.cpp
		COMMAND sed -e 's!{lang}!${lang}!' -e 's!{lang-title}!${lang_title}!' ../stencila/syntax-parser-parse.cxx > syntax-${lang}-parse.cpp
		WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
		DEPENDS stencila/syntax-parser-parse.cxx
	)
endfunction(generate_syntax)

generate_syntax(excel)

# Compile Stencila source files into a static library
add_library(stencila_sources STATIC 
	${SOURCES}
	${CMAKE_BINARY_DIR}/version.cpp
	${CMAKE_BINARY_DIR}/syntax-excel-lexer.cpp
	${CMAKE_BINARY_DIR}/syntax-excel-parser.cpp
	${CMAKE_BINARY_DIR}/syntax-excel-parse.cpp
)

# Static libraries combined into libstencila.a
set(BOOST_FILESYSTEM ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_filesystem.a)
set(BOOST_SYSTEM     ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_system.a)
set(BOOST_REGEX      ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_regex.a)
set(BOOST_UNITTEST   ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_unit_test_framework.a)
set(CMARK            ${CMAKE_BINARY_DIR}/requires/cmark/build/src/libcmark.a)
#set(CPR              ${CMAKE_BINARY_DIR}/requires/cpr/build/lib/libcpr.a)
set(LIBGIT2          ${CMAKE_BINARY_DIR}/requires/libgit2/build/libgit2.a)
set(LIBZIP           ${CMAKE_BINARY_DIR}/requires/libzip/build/lib/libzip.a)
set(PUGIXML          ${CMAKE_BINARY_DIR}/requires/pugixml/src/libpugixml.a)
set(TIDY             ${CMAKE_BINARY_DIR}/requires/tidy-html5/build/cmake/libtidys.a)


if(${CMAKE_SYSTEM_NAME} MATCHES "Linux" OR MSYS)
	# Use an ar script to combine static libraries
	# This is safer than extracting all the object files in each library (using `ar -x ...`)
	# and then combining them because it avoids the potential for name clashes causing some
	# object files to be over written
	file(WRITE ${CMAKE_BINARY_DIR}/ar-script.txt
		"CREATE libstencila.a\n"
		"ADDLIB " ${CMAKE_BINARY_DIR}/libstencila_sources.a "\n"
		"ADDLIB " ${BOOST_FILESYSTEM} "\n"
		"ADDLIB " ${BOOST_SYSTEM} "\n"
		"ADDLIB " ${BOOST_REGEX} "\n"
		"ADDLIB " ${CMARK} "\n"
		"ADDLIB " ${LIBGIT2} "\n"
		"ADDLIB " ${LIBZIP} "\n"		
		"ADDLIB " ${PUGIXML} "\n"
		"ADDLIB " ${TIDY} "\n"
		"SAVE\n"
		"END\n"
	)
	add_custom_command(
		OUTPUT ${CMAKE_BINARY_DIR}/libstencila.a
		COMMAND ar -M < ar-script.txt
		WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
		DEPENDS stencila_sources
		COMMENT "Combining static libraries into libstencila.a"
	)
endif()

if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
	# Combine static libraries with libtool
	add_custom_command(
		OUTPUT ${CMAKE_BINARY_DIR}/libstencila.a
		COMMAND libtool -static -o libstencila.a $<TARGET_FILE:stencila_sources> ${BOOST_FILESYSTEM} ${BOOST_SYSTEM} ${BOOST_REGEX} ${CMARK} ${LIBGIT2} ${LIBZIP} ${PUGIXML} ${TIDY}
		WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
		DEPENDS stencila_sources
	)
endif()

# Final combined static library
add_custom_target(stencila ALL DEPENDS ${CMAKE_BINARY_DIR}/libstencila.a)

# Test executable
add_executable(tests 
	${SOURCES}
	${TESTS}
	${CMAKE_BINARY_DIR}/version.cpp
	${CMAKE_BINARY_DIR}/syntax-excel-lexer.cpp
	${CMAKE_BINARY_DIR}/syntax-excel-parser.cpp
	${CMAKE_BINARY_DIR}/syntax-excel-parse.cpp
)
target_compile_options(tests PUBLIC
	-Wno-sign-compare
)
target_link_libraries(tests
	${BOOST_FILESYSTEM} ${BOOST_SYSTEM} ${BOOST_REGEX} ${CMARK} ${LIBGIT2} ${LIBZIP} ${PUGIXML} ${TIDY}
	${BOOST_UNITTEST}
	z curl ssl crypto
)
if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
	target_compile_options(tests PUBLIC
		-g -O0 --coverage
	)
	target_link_libraries(tests
		rt pthread
		gcov
	)
endif()
if(WIN32)
	target_link_libraries(tests
		ws2_32 mswsock ssh2
	)
endif()

set_target_properties(tests
    PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/tests
)
file(
	COPY tests/stencil-cila-html.txt
		 tests/stencil-cila-render.cila
		 tests/html-doc-1.html
	DESTINATION ${CMAKE_BINARY_DIR}/tests
)
