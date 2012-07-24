#pragma once

#include <vector>

namespace Stencila {
namespace Python {
namespace StandardLibraryBindings {
	
template<typename Type>
struct vector_to_list {
	static PyObject* convert(const std::vector<Type>& vec) {
		list* l = new list();
		for(size_t i = 0; i < vec.size(); i++) (*l).append(vec[i]);
		return l->ptr();
	}
};

void bind(void){
	to_python_converter<std::vector<std::string>, vector_to_list<std::string>>();
}

}}}