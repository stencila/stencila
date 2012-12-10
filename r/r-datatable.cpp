#include <stencila/datatable.hpp>
using namespace Stencila;

#include "r-extension.hpp"

STENCILA_R_FUNC Datatable_new(void){
	STENCILA_R_BEGIN
		return STENCILA_R_TO(Datatable,new Datatable);
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_rows(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Datatable>(self).rows()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_columns(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Datatable>(self).columns()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_dimensions(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Datatable>(self).dimensions()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_names(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Datatable>(self).names()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_type(SEXP self, SEXP column){
	STENCILA_R_BEGIN
		return wrap(
			from<Datatable>(self).type(as<unsigned int>(column)).name()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_types(SEXP self){
	STENCILA_R_BEGIN
		Rcpp::StringVector vec;
		BOOST_FOREACH(Datatype type,from<Datatable>(self).types()) vec.push_back(type.name());
		return vec;
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_index(SEXP self, SEXP columns){
	STENCILA_R_BEGIN
		from<Datatable>(self).index(
            as<std::vector<std::string>>(columns)
        );
		return nil;
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_head(SEXP self, SEXP rows){
	STENCILA_R_BEGIN
		Datatable dt = from<Datatable>(self);
        Datatable result;
        if(Rcpp::RObject(rows).isNULL()) result = dt.head();
        else result = dt.head(as<int>(rows));
        return STENCILA_R_TO(Datatable,new Datatable(result));
	STENCILA_R_END
}

STENCILA_R_FUNC Datatable_dataframe(SEXP self){
	STENCILA_R_BEGIN
		Datatable& dt = from<Datatable>(self);
		/*
		** See http://stackoverflow.com/questions/8631197/constructing-a-data-frame-in-rcpp
		*/
		//! @todo turn off stringsToFactors in creation of dataframe
		//! @todo create factors for ordinal and nominal mode columns
		Rcpp::List list;
		auto names = dt.names();
		auto types = dt.types();
		for(unsigned int column=0;column<dt.columns();column++){
			auto name = names[column];
			auto type = types[column];
			auto query = dt.dataset().cursor("SELECT "+name+" FROM "+dt.name());
			query.prepare();
			query.begin();
			if(type==Integer){
				Rcpp::IntegerVector vec;
				while(query.more()){
					vec.push_back(query.get<int>(0));
					query.next();
				}
				list.push_back(vec);
			}
			else if(type==Real){
				Rcpp::NumericVector vec;
				while(query.more()){
					vec.push_back(query.get<double>(0));
					query.next();
				}
				list.push_back(vec);
			}
			else if(type==Text){
				Rcpp::StringVector vec;
				while(query.more()){
					vec.push_back(query.get<std::string>(0));
					query.next();
				}
				list.push_back(vec);
			}
		}
		list.attr("names") = dt.names();
		return Rcpp::DataFrame(list);
	STENCILA_R_END
}
