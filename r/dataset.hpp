#include <stencila/dataset.hpp>
using namespace Stencila;

STENCILA_R_FUNC Dataset_new(void){
	STENCILA_R_BEGIN
		return STENCILA_R_TO(Dataset,new Dataset);
	STENCILA_R_END
}

STENCILA_R_FUNC Dataset_save(SEXP self, SEXP uri){
	STENCILA_R_BEGIN
		from<Dataset>(self).save(as<std::string>(uri));
		return nil;
	STENCILA_R_END
}

STENCILA_R_FUNC Dataset_tables(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Dataset>(self).tables()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Dataset_indices(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Dataset>(self).indices()
		);
	STENCILA_R_END
}

STENCILA_R_FUNC Dataset_execute(SEXP self, SEXP sql){
	STENCILA_R_BEGIN
		from<Dataset>(self).execute(as<std::string>(sql));
		return nil;
	STENCILA_R_END
}

STENCILA_R_FUNC Dataset_cursor(SEXP self, SEXP sql){
	STENCILA_R_BEGIN
		return STENCILA_R_TO(Datacursor,new Datacursor(
			from<Dataset>(self).cursor(as<std::string>(sql))
		));
	STENCILA_R_END
}

STENCILA_R_FUNC Dataset_table(SEXP self, SEXP table){
	STENCILA_R_BEGIN
		return STENCILA_R_TO(Datatable,new Datatable(
			from<Dataset>(self).table(as<std::string>(table))
		));
	STENCILA_R_END
}
