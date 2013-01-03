#include <stencila/dataquery.hpp>
using namespace Stencila;

#include "r-extension.hpp"

/*
When creating Dataquery elements in R garbage collection becomes an issue.
If a C++ Dataquery element points to another element that is garbage collected by R then Bad Things Happen (TM).
The following functions use RTTI to create a copy on a Dataquery element.
Using typeid() may be a faster alternative to using dynamic_cast<>().
*/
Element* clone(SEXP element){
    Element* o = &from<Element>(element);
    
    // If a null pointer then return immediately
    if(o==0) return 0;
    
    #define STENCILA_LOCAL(type) if(type* p = dynamic_cast<type*>(o)) return new type(*p);
    
    STENCILA_LOCAL(Constant<bool>)
    STENCILA_LOCAL(Constant<int>)
    STENCILA_LOCAL(Constant<double>)
    STENCILA_LOCAL(Constant<std::string>)
    
    STENCILA_LOCAL(Column)
    
    STENCILA_LOCAL(Negative)
    STENCILA_LOCAL(Positive)
    STENCILA_LOCAL(Not)
    
    STENCILA_LOCAL(Multiply)
    STENCILA_LOCAL(Divide)
    STENCILA_LOCAL(Add)
    STENCILA_LOCAL(Subtract)

    STENCILA_LOCAL(Equal)
    STENCILA_LOCAL(NotEqual)
    STENCILA_LOCAL(LessThan)
    STENCILA_LOCAL(LessEqual)
    STENCILA_LOCAL(GreaterThan)
    STENCILA_LOCAL(GreaterEqual)

    STENCILA_LOCAL(And)
    STENCILA_LOCAL(Or)
    
    STENCILA_LOCAL(Call)
    STENCILA_LOCAL(Aggregate)
    STENCILA_LOCAL(As)
    STENCILA_LOCAL(Distinct)
    STENCILA_LOCAL(All)
    STENCILA_LOCAL(Where)
    STENCILA_LOCAL(By)
    STENCILA_LOCAL(Having)
    STENCILA_LOCAL(Order)
    STENCILA_LOCAL(Limit)
    STENCILA_LOCAL(Offset)
    
    #undef STENCILA_LOCAL
    
    STENCILA_THROW(Exception,"Unhandled type");
    return 0;
}

STENCILA_R_FUNC Element_null(SEXP self){
    STENCILA_R_BEGIN
        return to(static_cast<Element*>(0),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_dql(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Element>(self).dql()
        );
    STENCILA_R_END
}

//! @name Constants
//! @{

STENCILA_R_FUNC Element_Logical(SEXP value){
    STENCILA_R_BEGIN
        return to(new Constant<bool>(as<bool>(value)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Integer(SEXP value){
    STENCILA_R_BEGIN
        return to(new Constant<int>(as<int>(value)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Numeric(SEXP value){
    STENCILA_R_BEGIN
        return to(new Constant<double>(as<double>(value)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_String(SEXP value){
    STENCILA_R_BEGIN
        return to(new Constant<std::string>(as<std::string>(value)),"Element");
    STENCILA_R_END
}

//! @}

STENCILA_R_FUNC Element_Column(SEXP str){
    STENCILA_R_BEGIN
        return to(new Column(
            as<std::string>(str)
        ),"Element");
    STENCILA_R_END
}

//! @name Unary operators
//! @{

#define STENCILA_LOCAL(name) \
STENCILA_R_FUNC Element_##name(SEXP element){ \
    STENCILA_R_BEGIN \
        return to(new name(clone(element)),"Element"); \
    STENCILA_R_END \
}

STENCILA_LOCAL(Negative)
STENCILA_LOCAL(Positive)
STENCILA_LOCAL(Not)

#undef STENCILA_LOCAL

//! @}

//! @name Binary operators
//! @{

#define STENCILA_LOCAL(name) \
STENCILA_R_FUNC Element_##name(SEXP left, SEXP right){ \
    STENCILA_R_BEGIN \
        return to(new name(clone(left),clone(right)),"Element"); \
    STENCILA_R_END \
}

STENCILA_LOCAL(Multiply)
STENCILA_LOCAL(Divide)
STENCILA_LOCAL(Add)
STENCILA_LOCAL(Subtract)

STENCILA_LOCAL(Equal)
STENCILA_LOCAL(NotEqual)
STENCILA_LOCAL(LessThan)
STENCILA_LOCAL(LessEqual)
STENCILA_LOCAL(GreaterThan)
STENCILA_LOCAL(GreaterEqual)

STENCILA_LOCAL(And)
STENCILA_LOCAL(Or)

#undef STENCILA_LOCAL

//! @}

STENCILA_R_FUNC Element_Call(SEXP name, SEXP arguments){
    STENCILA_R_BEGIN
        Call* c = new Call(as<std::string>(name));
        Rcpp::List pointers(arguments);
        for(Rcpp::List::iterator i=pointers.begin();i!=pointers.end();i++){
            c->append(clone(*i));
        }
        return to(c,"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Aggregate(SEXP name, SEXP element){
    STENCILA_R_BEGIN
        return to(new Aggregate(
            as<std::string>(name),
            clone(element)
        ),"Element");
    STENCILA_R_END
}

//! @name Standard directives
//! @{

STENCILA_R_FUNC Element_As(SEXP element,SEXP name){
    STENCILA_R_BEGIN
        return to(new As(
            clone(element),
            as<std::string>(name)
        ),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Distinct(void){
    STENCILA_R_BEGIN
        return to(new Distinct,"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_All(void){
    STENCILA_R_BEGIN
        return to(new All,"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Where(SEXP element){
    STENCILA_R_BEGIN
        return to(new Where(clone(element)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_By(SEXP element){
    STENCILA_R_BEGIN
        return to(new By(clone(element)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Having(SEXP element){
    STENCILA_R_BEGIN
        return to(new Having(clone(element)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Order(SEXP element){
    STENCILA_R_BEGIN
        return to(new Order(clone(element)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Limit(SEXP number){
    STENCILA_R_BEGIN
        return to(new Limit(as<unsigned int>(number)),"Element");
    STENCILA_R_END
}

STENCILA_R_FUNC Element_Offset(SEXP number){
    STENCILA_R_BEGIN
        return to(new Offset(as<unsigned int>(number)),"Element");
    STENCILA_R_END
}

//! @}

//! @name Combiners
//! @{

STENCILA_R_FUNC Element_Top(SEXP by, SEXP element, SEXP number){
    STENCILA_R_BEGIN
        return to(new Top(
            clone(by),
            clone(element),
            as<unsigned int>(number)
        ),"Element");
    STENCILA_R_END
}

//! @}

//! @name Margins
//! @{

STENCILA_R_FUNC Element_Margin(SEXP element){
    STENCILA_R_BEGIN
        return to(new Margin(clone(element)),"Element");
    STENCILA_R_END
}

//! @}

//! @name Adjusters
//! @{

STENCILA_R_FUNC Element_Proportion(SEXP value, SEXP by){
    STENCILA_R_BEGIN
        Proportion* prop = new Proportion(clone(value));
        Element* b = clone(by);
        if(b) prop->bys_append(b);
        return to(prop,"Element");
    STENCILA_R_END
}

//! @}

//! @name Reshapers
//! @{
//! @}

//! @name Dataquery
//! @{

STENCILA_R_FUNC Dataquery_new(SEXP elements){
    STENCILA_R_BEGIN
        Dataquery* q = new Dataquery;
        Rcpp::List pointers(elements);
        for(Rcpp::List::iterator i=pointers.begin();i!=pointers.end();i++){
            q->append(clone(*i));
        }
        return to(q,"Dataquery");
    STENCILA_R_END
}

STENCILA_R_FUNC Dataquery_new_noargs(void){
    STENCILA_R_BEGIN
        return to(new Dataquery,"Dataquery");
    STENCILA_R_END
}

STENCILA_R_FUNC Dataquery_dql(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Dataquery>(self).dql()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Dataquery_execute(SEXP self, SEXP datatable){
    STENCILA_R_BEGIN
        Datatable result = from<Dataquery>(self).execute(
            from<Datatable>(datatable)
        );
        return to(new Datatable(result),"Datatable");
    STENCILA_R_END
}

//! @}
