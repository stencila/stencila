#pragma once

#include <string>

#include <boost/python.hpp>
using namespace boost::python;
namespace python = boost::python;

// For use of str() it seems necessary to do the following
namespace boost {
namespace python {
    using self_ns::str;
}
}

#include <stencila/stencila.hpp>