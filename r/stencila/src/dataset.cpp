#include "../../../cpp/dataset.hpp"
using namespace Stencila;

EXPORT SEXP Dataset_new(void){
	BEGIN
		return TO(Dataset,new Dataset);
	END
}

EXPORT SEXP Dataset_save(SEXP self, SEXP uri){
	BEGIN
		from<Dataset>(self).save(as<std::string>(uri));
		return NIL;
	END
}

EXPORT SEXP Dataset_tables(SEXP self){
	BEGIN
		return wrap(
			from<Dataset>(self).tables()
		);
	END
}

EXPORT SEXP Dataset_indices(SEXP self){
	BEGIN
		return wrap(
			from<Dataset>(self).indices()
		);
	END
}

EXPORT SEXP Dataset_execute(SEXP self, SEXP sql){
	BEGIN
		from<Dataset>(self).execute(as<std::string>(sql));
		return NIL;
	END
}

EXPORT SEXP Dataset_cursor(SEXP self, SEXP sql){
	BEGIN
		return TO(Datacursor,new Datacursor(
			from<Dataset>(self).cursor(as<std::string>(sql))
		));
	END
}

EXPORT SEXP Dataset_table(SEXP self, SEXP table){
	BEGIN
		return TO(Datatable,new Datatable(
			from<Dataset>(self).table(as<std::string>(table))
		));
	END
}
