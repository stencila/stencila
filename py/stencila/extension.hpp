#pragma once

#include <string>

#include <boost/python.hpp>
namespace bp = boost::python;

#include <stencila/stencila.hpp>

std::string Stencila_version(void){
	return Stencila::version;
}
