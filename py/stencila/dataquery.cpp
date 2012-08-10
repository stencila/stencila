#pragma once

#include <stencila/dataquery.hpp>

namespace Dataquery_ {

using namespace Stencila;

//! @brief Convert a Python object into a Dataquery element
//!
//! This function is a "raw_function" which recieves Python objects.
//! Its job is to convert those Python objects into Stencila Dataquery elements
//! For example, a Python integer is converted into a Constant<int>
const Expression* wrap(const object& o){
    //Boost.python's extract<float>(o).check() returns true even if o is an integer.
    //So have to use the Python PyXXX_Check functions to determine type
    const PyObject* p = o.ptr();
    if(PyBool_Check(p)) return new Constant<bool>(extract<bool>(o));
    if(PyInt_Check(p)) return new Constant<int>(extract<int>(o));
    if(PyFloat_Check(p)) return new Constant<float>(extract<float>(o));
    if(PyString_Check(p)) return new Constant<std::string>(extract<std::string>(o));
    
    //If the object is a Dataquery element then just return it
    extract<const Expression*> e(o);
    if(e.check()) return e();
    
    //Any othe object type is converted to a string
    //! @todo Obtain the __repr__ or __str__ of the object
    return new Constant<std::string>(boost::lexical_cast<std::string>(p));
}

#define UNOP(name,type) \
    type name(const Expression& self){ \
        return type(&self); \
    }

UNOP(__neg__,Negative)
UNOP(__pos__,Positive)

#undef UNOP

#define BINOP(name,type) \
    type name(const Expression& self, const object& other){ \
        return type(&self,wrap(other)); \
    }

BINOP(__eq__,Equal)
BINOP(__ne__,NotEqual)
BINOP(__lt__,LessThan)
BINOP(__le__,LessEqual)
BINOP(__gt__,GreaterThan)
BINOP(__ge__,GreaterEqual)

BINOP(__add__,Add)
BINOP(__sub__,Subtract)
BINOP(__mul__,Multiply)
BINOP(__div__,Divide)

BINOP(__and__,And)
BINOP(__or__,Or)

#undef BINOP

void bind(void){

    class_<Element>("Element")
        .def("dql",&Element::dql)
        .def("sql",&Element::sql)
    ;
    
    class_<Expression,bases<Element>>("Expression")
        
        #define OP(name) .def(#name,name)
        
        OP(__pos__)
        OP(__neg__)

        OP(__eq__)
        OP(__ne__)
        OP(__lt__)
        OP(__le__)
        OP(__gt__)
        OP(__ge__)

        OP(__add__)
        OP(__sub__)
        OP(__mul__)
        OP(__div__)
        
        OP(__and__)
        OP(__or__)
        
        #undef OP
    ;
    
    class_<Column,bases<Expression>>("Column",init<std::string>());
    
    //Define Python classes for each unary operator
    #define UNOP(name) class_<name,bases<Expression>>(#name);

    UNOP(Negative)
    UNOP(Positive)
    
    #undef UNOP
    
    //Define Python classes for each binary operator
    #define BINOP(name) class_<name,bases<Expression>>(#name);

    BINOP(Multiply)
    BINOP(Divide)
    BINOP(Add)
    BINOP(Subtract)

    BINOP(Equal)
    BINOP(NotEqual)
    BINOP(LessThan)
    BINOP(LessEqual)
    BINOP(GreaterThan)
    BINOP(GreaterEqual)

    BINOP(And)
    BINOP(Or)
    
    #undef BINOP
}

}