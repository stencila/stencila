#include <stencila/dataquery.hpp>
using namespace Stencila;

#include "r-extension.hpp"

/*
When creating Dataquery expressions in R garbage collection becomes an issue.
If a C++ Dataquery element points to another element that is garbage collected by R then bad thuings happen.
The following functions use RTTI to create a copy on a Dataquery element.
Using typeid() may be a faster alternative to using dynamic_cast<>().
*/
Element* cloneEl(SEXP element){
    Element* o = &from<Element>(element);
    
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
//Some Dataquery element expect an Expression, not just an element
//so this function is required for now
//! @todo refactor the various dataquery elements so they always point to other Elements*
Expression* cloneEx(SEXP element){
    return static_cast<Expression*>(cloneEl(element));
}

STENCILA_R_FUNC Expression_dql(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Expression>(self).dql()
        );
    STENCILA_R_END
}

//////////////////
// Constants

STENCILA_R_FUNC Expression_Logical(SEXP value){
	STENCILA_R_BEGIN
		return to(new Constant<bool>(as<bool>(value)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Integer(SEXP value){
	STENCILA_R_BEGIN
		return to(new Constant<int>(as<int>(value)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Numeric(SEXP value){
	STENCILA_R_BEGIN
		return to(new Constant<double>(as<double>(value)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_String(SEXP value){
	STENCILA_R_BEGIN
		return to(new Constant<std::string>(as<std::string>(value)),"Expression");
	STENCILA_R_END
}

//////////////////
// Column

STENCILA_R_FUNC Expression_Column(SEXP str){
	STENCILA_R_BEGIN
		return to(new Column(
            as<std::string>(str)
        ),"Expression");
	STENCILA_R_END
}

//////////////////
// Unary operators

#define STENCILA_LOCAL(name) \
STENCILA_R_FUNC Expression_##name(SEXP expr){ \
	STENCILA_R_BEGIN \
		return to(new name(cloneEx(expr)),"Expression"); \
	STENCILA_R_END \
}

STENCILA_LOCAL(Negative)
STENCILA_LOCAL(Positive)
STENCILA_LOCAL(Not)

#undef STENCILA_LOCAL

//////////////////
// Binary operators

#define STENCILA_LOCAL(name) \
STENCILA_R_FUNC Expression_##name(SEXP left, SEXP right){ \
	STENCILA_R_BEGIN \
		return to(new name(cloneEx(left),cloneEx(right)),"Expression"); \
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

STENCILA_R_FUNC Expression_Call(SEXP name, SEXP arguments){
	STENCILA_R_BEGIN
        Call* c = new Call(as<std::string>(name));
        Rcpp::List pointers(arguments);
        for(Rcpp::List::iterator i=pointers.begin();i!=pointers.end();i++){
            c->append(cloneEx(*i));
        }
		return to(c,"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Aggregate(SEXP name, SEXP expr){
	STENCILA_R_BEGIN
        return to(new Aggregate(
            as<std::string>(name),
            cloneEx(expr)
        ),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_As(SEXP element,SEXP name){
	STENCILA_R_BEGIN
		return to(new As(
            cloneEl(element),
            as<std::string>(name)
        ),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Distinct(void){
	STENCILA_R_BEGIN
		return to(new Distinct,"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_All(void){
	STENCILA_R_BEGIN
		return to(new All,"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Where(SEXP expr){
	STENCILA_R_BEGIN
		return to(new Where(cloneEx(expr)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_By(SEXP expr){
	STENCILA_R_BEGIN
		return to(new By(cloneEl(expr)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Having(SEXP expr){
	STENCILA_R_BEGIN
		return to(new Having(cloneEx(expr)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Order(SEXP expr){
	STENCILA_R_BEGIN
		return to(new Order(cloneEx(expr)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Limit(SEXP expr){
	STENCILA_R_BEGIN
		return to(new Limit(cloneEx(expr)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_Offset(SEXP expr){
	STENCILA_R_BEGIN
		return to(new Offset(cloneEx(expr)),"Expression");
	STENCILA_R_END
}

//////////////////

STENCILA_R_FUNC Dataquery_new(SEXP elements){
	STENCILA_R_BEGIN
        Dataquery* q = new Dataquery;
        Rcpp::List pointers(elements);
        for(Rcpp::List::iterator i=pointers.begin();i!=pointers.end();i++){
            q->append(cloneEx(*i));
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