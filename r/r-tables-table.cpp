#include <stencila/datatypes.hpp>
using namespace Stencila;
#include <stencila/tables/table.hpp>
using namespace Stencila::Tables;

#include "r-extension.hpp"

STENCILA_R_FUNC Table_new(void){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Table,new Table);
    STENCILA_R_END
}

STENCILA_R_FUNC Table_name(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).name()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_rows(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).rows()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_columns(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).columns()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_dimensions(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).dimensions()
        );
    STENCILA_R_END
}


STENCILA_R_FUNC Table_label(SEXP self, SEXP column){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).label(as<unsigned int>(column))
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_labels(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).labels()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_type(SEXP self, SEXP column){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).type(as<unsigned int>(column)).name()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_types(SEXP self){
    STENCILA_R_BEGIN
        Rcpp::StringVector vec;
        BOOST_FOREACH(Datatype type,from<Table>(self).types()) vec.push_back(type.name());
        return vec;
    STENCILA_R_END
}

STENCILA_R_FUNC Table_index(SEXP self, SEXP columns){
    STENCILA_R_BEGIN
        from<Table>(self).index(
            as<std::vector<std::string>>(columns)
        );
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Table_indices(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Table>(self).indices()
        );
    STENCILA_R_END
}

STENCILA_R_FUNC Table_head(SEXP self, SEXP rows){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Table,new Table(
            from<Table>(self).head(as<int>(rows))
        ));
    STENCILA_R_END
}

STENCILA_R_FUNC Table_tail(SEXP self, SEXP rows){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Table,new Table(
            from<Table>(self).tail(as<int>(rows))
        ));
    STENCILA_R_END
}

STENCILA_R_FUNC Table_value(SEXP self, SEXP row, SEXP col){
    STENCILA_R_BEGIN
        Table dt = from<Table>(self);
        int r = as<int>(row);
        int c = as<int>(col);
        switch(dt.type(c).code){
            case 'n': return wrap(dt.value<std::string>(r,c));
            case 'i': return wrap(dt.value<int>(r,c));
            case 'r': return wrap(dt.value<double>(r,c));
            default : return wrap(dt.value<std::string>(r,c));
        }
    STENCILA_R_END
}

STENCILA_R_FUNC Table_to_dataframe(SEXP self){
    STENCILA_R_BEGIN
        Table& table = from<Table>(self);
        /*
        ** See http://stackoverflow.com/questions/8631197/constructing-a-data-frame-in-rcpp
        */
        //! @todo turn off stringsToFactors in creation of dataframe
        //! @todo create factors for ordinal and nominal mode columns
        Rcpp::List list;
        auto labels = table.labels();
        auto types = table.types();
        for(unsigned int column=0;column<table.columns();column++){
            auto label = labels[column];
            auto type = types[column];
            auto query = table.cursor("SELECT \""+label+"\" FROM \""+table.name()+"\"");
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
        list.attr("names") = table.labels();
        return Rcpp::DataFrame(list);
    STENCILA_R_END
}

STENCILA_R_FUNC Table_from_dataframe(SEXP dataframe){
    STENCILA_R_BEGIN
        Rcpp::DataFrame df(dataframe);
        Table& dt = *new Table;
        
        // Get the column names and types of the data.frame
        //! @todo How to get the df column types? Does it matter?
        std::vector<std::string> names = as<std::vector<std::string>>(df.names());
        for(std::string name : names){
            dt.add(name, Text);
        }
        
        // R stores data.frames as a vector of vectors (one for each column). So it is necessary to extract each 
        // column individually and then do the row insert.
        std::vector<std::vector<std::string>> columns;
        for(std::string name : names){
            auto column = df[name];
            if(Rf_isString(column)) columns.push_back(as<std::vector<std::string>>(column));
        }
        
        //Work out the number of rows
        int nrow = 0;
        if(columns.size()>0) nrow = columns.begin()->size();
        
        // Insert on a row by row basis
        for(int row=0;row<nrow;row++){
            std::vector<std::string> values;
            for(std::vector<std::string> column : columns) values.push_back(column[row]);
            dt.append(values);
        }
        
        return STENCILA_R_TO(Table,&dt);
    STENCILA_R_END
}
