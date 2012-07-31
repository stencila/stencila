#include "../cpp/datatable.hpp"
using namespace Stencila;

EXPORT SEXP Datatable_new(void){
	BEGIN
		return TO(Datatable,new Datatable);
	END
}

EXPORT SEXP Datatable_rows(SEXP self){
	BEGIN
		return wrap(
			from<Datatable>(self).rows()
		);
	END
}

EXPORT SEXP Datatable_columns(SEXP self){
	BEGIN
		return wrap(
			from<Datatable>(self).columns()
		);
	END
}

EXPORT SEXP Datatable_dimensions(SEXP self){
	BEGIN
		return wrap(
			from<Datatable>(self).dimensions()
		);
	END
}

EXPORT SEXP Datatable_names(SEXP self){
	BEGIN
		return wrap(
			from<Datatable>(self).names()
		);
	END
}

EXPORT SEXP Datatable_type(SEXP self, SEXP column){
	BEGIN
		return wrap(
			from<Datatable>(self).type(as<unsigned int>(column)).name()
		);
	END
}

EXPORT SEXP Datatable_types(SEXP self){
	BEGIN
		Rcpp::StringVector vec;
		BOOST_FOREACH(Datatype type,from<Datatable>(self).types()) vec.push_back(type.name());
		return vec;
	END
}

EXPORT SEXP Datatable_dataframe(SEXP self){
	BEGIN
		Datatable& dt = from<Datatable>(self);
		/*
		** See http://stackoverflow.com/questions/8631197/constructing-a-data-frame-in-rcpp
		*/
		//! @todo turn off stringsToFactors in creartion of dataframe
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
	END
}
