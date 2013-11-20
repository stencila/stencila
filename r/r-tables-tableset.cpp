#include <stencila/tables/tableset.hpp>
#include <stencila/tables/table.hpp>
using namespace Stencila::Tables;

#include "r-extension.hpp"

STENCILA_R_FUNC Tableset_new(SEXP uri){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Tableset,
            new Tableset(as<std::string>(uri))
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_uri(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Tableset>(self).uri()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_tables(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Tableset>(self).tables()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_index(SEXP self, SEXP table, SEXP columns){
    STENCILA_R_BEGIN
        from<Tableset>(self).index(
            as<std::string>(table),
            as<std::vector<std::string>>(columns)
        );
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_indices(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Tableset>(self).indices()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_load(SEXP self, SEXP name, SEXP path){
    STENCILA_R_BEGIN
        from<Tableset>(self).load(as<std::string>(name),as<std::string>(path));
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_save(SEXP self, SEXP uri){
    STENCILA_R_BEGIN
        from<Tableset>(self).save(as<std::string>(uri));
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_execute(SEXP self, SEXP sql){
    STENCILA_R_BEGIN
        from<Tableset>(self).execute(as<std::string>(sql));
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_select(SEXP self, SEXP sql){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Table,new Table(
            from<Tableset>(self).select(as<std::string>(sql))
        ));
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_cursor(SEXP self, SEXP sql){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Cursor,new Cursor(
            from<Tableset>(self).cursor(as<std::string>(sql))
        ));
    STENCILA_R_END
}

STENCILA_R_FUNC Tableset_table(SEXP self, SEXP table){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Table,new Table(
            from<Tableset>(self).table(as<std::string>(table))
        ));
    STENCILA_R_END
}


