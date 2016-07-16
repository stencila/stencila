cmake_minimum_required(VERSION 2.6)
project(Stencila)


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

# Compile Stencila source files into a static library
file(GLOB_RECURSE SOURCES stencila/*.cpp)
add_library(stencila_sources STATIC 
	${SOURCES}
	${CMAKE_BINARY_DIR}/version.cpp
)

# Compiler options
target_compile_options(stencila_sources PUBLIC
	-std=c++11 -O2
	-Wall -Wno-unused-variable
)
if ("${CMAKE_CXX_COMPILER_ID}" MATCHES "Clang")
	target_compile_options(stencila_sources PUBLIC
		-Wno-unknown-pragmas # prevents many warnings
		-Wno-missing-braces # prevents unecessary warnings, see http://stackoverflow.com/a/13905432/4625911
	)
endif()
if(NOT WIN32)
	target_compile_options(stencila_sources PUBLIC
		-fPIC # because eventually linked into shared dynamic libraries
	)	
endif()

# Include directories
target_include_directories(stencila_sources PUBLIC
	.
	${CMAKE_BINARY_DIR}/requires/boost
	${CMAKE_BINARY_DIR}/requires/cmark/src ${CMAKE_BINARY_DIR}/requires/cmark/build/src
	${CMAKE_BINARY_DIR}/requires/cpr/include
	${CMAKE_BINARY_DIR}/requires/jsoncpp/dist
	${CMAKE_BINARY_DIR}/requires/libgit2/include
	${CMAKE_BINARY_DIR}/requires/libzip/lib ${CMAKE_BINARY_DIR}/requires/libzip/build
	${CMAKE_BINARY_DIR}/requires/pugixml/src
	${CMAKE_BINARY_DIR}/requires/tidy-html5/include
	${CMAKE_BINARY_DIR}/requires/websocketpp
)


# Static libraries combined into libstencila.a
set(BOOST_FILESYSTEM ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_filesystem.a)
set(BOOST_SYSTEM     ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_system.a)
set(BOOST_REGEX      ${CMAKE_BINARY_DIR}/requires/boost/lib/libboost_regex.a)
set(CMARK            ${CMAKE_BINARY_DIR}/requires/cmark/build/src/libcmark.a)
set(CPR              ${CMAKE_BINARY_DIR}/requires/cpr/build/lib/libcpr.a)
set(LIBGIT2          ${CMAKE_BINARY_DIR}/requires/libgit2/build/libgit2.a)
set(LIBZIP           ${CMAKE_BINARY_DIR}/requires/libzip/build/lib/libzip.a)
set(PUGIXML          ${CMAKE_BINARY_DIR}/requires/pugixml/src/libpugixml.a)
set(TIDY             ${CMAKE_BINARY_DIR}/requires/tidy-html5/build/cmake/libtidys.a)

if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
	# Use an ar script to combine static libraries
	# This is safer than extracting all the object files in each library (using `ar -x ...`)
	# and then combining them because it avoids the potential for name clashes causing some
	# object files to be over written
	file(WRITE ${CMAKE_BINARY_DIR}/ar-script.txt
		"CREATE libstencila.a\n"
		"ADDLIB " ${CMAKE_BINARY_DIR}/libstencila_sources.a "\n"
		"ADDLIB " ${BOOST_FILESYSTEM} "\n"
		"ADDLIB " ${BOOST_SYSTEM} "\n"
		"ADDLIB " ${CMARK} "\n"
		"ADDLIB " ${CPR} "\n"
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
		COMMAND libtool -static -o libstencila.a $<TARGET_FILE:stencila_sources> ${BOOST_FILESYSTEM} ${BOOST_SYSTEM} ${BOOST_REGEX} ${CMARK} ${CPR} ${LIBGIT2} ${LIBZIP} ${PUGIXML} ${TIDY}	
		WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
		DEPENDS stencila_sources
	)
endif()

# Final target static library
add_custom_target(stencila ALL DEPENDS ${CMAKE_BINARY_DIR}/libstencila.a)
