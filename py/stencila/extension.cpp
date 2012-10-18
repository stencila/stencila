/*
**
**	http://www.boost.org/doc/libs/1_50_0/libs/python/doc/tutorial/doc/html/python/exposing.html

*/

#include <boost/python.hpp>
#include <boost/python/slice.hpp>
#include <boost/python/raw_function.hpp>

namespace boost {
namespace python {
	using self_ns::str;
}
}

using namespace boost::python;
namespace bp = boost::python;

#include "standard_library.cpp"
#include "exception.cpp"
#include "datatype.cpp"
#include "dataset.cpp"
#include "datatable.cpp"
#include "dataquery.cpp"

#include "stencil.cpp"

BOOST_PYTHON_MODULE(extension){
	using namespace Stencila::Python;
	
	StandardLibraryBindings::bind();
	ExceptionBindings::bind();
	DatatypeBindings::bind();
	DatasetBindings::bind();
	DatatableBindings::bind();
	
	Dataquery_::bind();
    
    Stencil_::bind();
}
