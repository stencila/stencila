#pragma once

#include <string>

#include <boost/python.hpp>
using namespace boost::python;
namespace bp = boost::python;

#include <stencila/stencila.hpp>

std::string Stencila_version(void){
	return Stencila::version;
}
