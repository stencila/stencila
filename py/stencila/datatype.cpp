#pragma once

#include <stencila/datatypes.hpp>

namespace Stencila {
namespace Python {
namespace DatatypeBindings {
	
void bind(void){
	class_<Datatype,bases<>>("Datatype")
		.def(init<char>())
	;
}

}}}