#include <stencila/dataset.hpp>
#include <stencila/datatable.hpp>
using namespace Stencila;

#include "r-extension.hpp"

STENCILA_R_FUNC Dataset_new(SEXP uri){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Dataset,
            new Dataset(as<std::string>(uri))
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Dataset_uri(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Dataset>(self).uri()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Dataset_tables(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Dataset>(self).tables()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Dataset_index(SEXP self, SEXP table, SEXP columns){
    STENCILA_R_BEGIN
        from<Dataset>(self).index(
            as<std::string>(table),
            as<std::vector<std::string>>(columns)
        );
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Dataset_indices(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Dataset>(self).indices()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Dataset_load(SEXP self, SEXP name, SEXP path){
    STENCILA_R_BEGIN
        from<Dataset>(self).load(as<std::string>(name),as<std::string>(path));
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Dataset_save(SEXP self, SEXP uri){
    STENCILA_R_BEGIN
        from<Dataset>(self).save(as<std::string>(uri));
        return nil;
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
