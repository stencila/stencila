#include "stencila.hpp"
#include <stencila/dataquery.hpp>
using namespace Stencila;

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
		return to(new name(&from<Expression>(expr)),"Expression"); \
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
		return to(new name(&from<Expression>(left),&from<Expression>(right)),"Expression"); \
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

//////////////////
// Clauses

STENCILA_R_FUNC Expression_Call(SEXP name, SEXP arguments){
	STENCILA_R_BEGIN
        Call* c = new Call(as<std::string>(name));
        Rcpp::List pointers(arguments);
        for(Rcpp::List::iterator i=pointers.begin();i!=pointers.end();i++){
            c->append(&from<Expression>(*i));
        }
		return to(c,"Expression");
	STENCILA_R_END
}

//////////////////
// Clauses

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
		return to(new Where(&from<Expression>(expr)),"Expression");
	STENCILA_R_END
}

STENCILA_R_FUNC Expression_By(SEXP expr){
	STENCILA_R_BEGIN
		return to(new By(&from<Expression>(expr)),"Expression");
	STENCILA_R_END
}

//////////////////

STENCILA_R_FUNC Dataquery_new(SEXP elements){
	STENCILA_R_BEGIN
        Dataquery* q = new Dataquery;
        Rcpp::List pointers(elements);
        for(Rcpp::List::iterator i=pointers.begin();i!=pointers.end();i++){
            q->append(&from<Expression>(*i));
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

STENCILA_R_FUNC Dataquery_sql(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
            from<Dataquery>(self).sql()
        );
	STENCILA_R_END
}
