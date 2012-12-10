#include <vector>

#include "py-extension.hpp"

extern void Exception_define(void);
extern void Datatype_define(void);
extern void Dataset_define(void);
extern void Datatable_define(void);
extern void Dataquery_define(void);
extern void Stencil_define(void);

template<typename Type>
struct vector_to_list {
	static PyObject* convert(const std::vector<Type>& vec) {
		list* l = new list();
		for(size_t i = 0; i < vec.size(); i++) (*l).append(vec[i]);
		return l->ptr();
	}
};

BOOST_PYTHON_MODULE(extension){
    
    to_python_converter<std::vector<std::string>, vector_to_list<std::string>>();

    Exception_define();
    Datatype_define();
    Dataset_define();
    Datatable_define();
    Dataquery_define();
    Stencil_define();
}
