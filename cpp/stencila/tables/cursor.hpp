//! @file cursor.hpp
//! @brief Definition of class Cursor
//! @author Nokome Bentley

#pragma once

#include <stencila/exception.hpp>
#include <stencila/tables/sqlite.hpp>
#include <stencila/datatypes.hpp>

namespace Stencila {
namespace Tables {

class Cursor {

private:
    
    sqlite3* db_;
    std::string sql_;
    sqlite3_stmt* stmt_;
    bool begun_;
    bool more_;

public:

    Cursor(sqlite3* db, const std::string& sql):
        db_(db),
        sql_(sql),
        stmt_(0),
        begun_(false),
        more_(false){
    }

    //! @brief
    //! @param db
    //! @param sql
    //! @param pars
    //! @return
    template<typename... Parameters>
    Cursor(sqlite3* db, const std::string& sql, Parameters&... pars):
        db_(db),
        sql_(sql),
        stmt_(0),
        begun_(false),
        more_(false){
        prepare();
        use(pars...);
    }

    //! @brief
    //! @return
    ~Cursor(void){
        if(stmt_){
            STENCILA_SQLITE_TRY(db_,sqlite3_finalize(stmt_));
        }
    }

    //! @brief
    //! @return
    const std::string& sql(void) const {
        return sql_;
    }

    //! @brief
    //! @return
    bool more(void) const {
        return more_;
    }

    //! @brief
    //! @return
    Cursor& prepare(void){
        STENCILA_SQLITE_TRY(db_,sqlite3_prepare_v2(db_, sql_.c_str(), -1, &stmt_, 0));
        return *this;
    }

    //! @name Parameter binding methods
    //! @brief Bind values to parameters in SQL
    //! @{
    //! @warning Calls to Cursor::bind methods must be preceded by a call to Cursor::prepare

    Cursor& bind(unsigned int index){
        STENCILA_SQLITE_TRY(db_,sqlite3_bind_null(stmt_,index));
        return *this;
    }

    //! @brief
    //! @param value
    //! @return
    Cursor& bind(unsigned int index,const int& value){
        STENCILA_SQLITE_TRY(db_,sqlite3_bind_int(stmt_,index,value));
        return *this;
    }

    //! @brief
    //! @param value
    //! @return
    Cursor& bind(unsigned int index,const unsigned int& value){
        int value_int = boost::numeric_cast<int>(value);
        STENCILA_SQLITE_TRY(db_,sqlite3_bind_int(stmt_,index,value_int));
        return *this;
    }

    //! @brief
    //! @param value
    //! @return
    Cursor& bind(unsigned int index,const long int& value){
        uint64_t value_int = boost::numeric_cast<uint64_t>(value);
        STENCILA_SQLITE_TRY(db_,sqlite3_bind_int64(stmt_,index,value_int));
        return *this;
    }

    //! @brief
    //! param value
    //! @return
    Cursor& bind(unsigned int index,const double& value){
        STENCILA_SQLITE_TRY(db_,sqlite3_bind_double(stmt_,index,value));
        return *this;
    }

    //! @brief
    //! @param index
    //! @param std
    //! @param value
    //! @return
    Cursor& bind(unsigned int index,const std::string& value){
        // SQLITE_TRANSIENT is used so that SQLite makes a copy of the string data
        // See http://stackoverflow.com/a/10132955
        STENCILA_SQLITE_TRY(db_,sqlite3_bind_text(stmt_,index,value.data(),value.length(),SQLITE_TRANSIENT));
        return *this;
    }

    //! @brief
    //! @return
    template<
        typename Parameter,
        typename... Parameters
    >
    Cursor& use(const Parameter& par, const Parameters&... pars){
        int count = sqlite3_bind_parameter_count(stmt_);
        int index = count - sizeof...(Parameters);
        bind(index,par);
        use(pars...);
        return *this;
    }

    //! @brief
    //! @return
    Cursor& use(void){
        return *this;
    }

    //! @}

    void reset(void){
        STENCILA_SQLITE_TRY(db_,sqlite3_clear_bindings(stmt_));
        STENCILA_SQLITE_TRY(db_,sqlite3_reset(stmt_));
        begun_ = false;
    }

    //! @brief
    //! @return
    void begin(void){
        if(not begun_) {
            prepare();
            next();
            begun_ = true;
        }
    }

    void execute(void){
        //If a statement has already been prepared then sqlite3_step that...
        //sqlite3_step does not always return SQLITE_OK on success so do not wrap it in STENCILA_SQLITE_TRY
        if(stmt_){
            sqlite3_step(stmt_);
        }
        //Otherwise use the sqlite3_exec shortcut function to prepare, step and finalise in one
        else {
            STENCILA_SQLITE_TRY(db_,sqlite3_exec(db_,sql_.c_str(),0,0,0));
        }
    }

    //! @brief
    //! @param pars
    //! @return
    template<typename... Parameters>
    void execute(const Parameters... pars){
        prepare();
        use(pars...);
        execute();
    }

    //! @warning Must be preceded by a call to Cursor::prepare
    void next(void){
        int code = sqlite3_step(stmt_);
        if(code==SQLITE_ROW) {
            // sqlite3_step() has another row ready
            more_ =  true;
        }
        else if(code==SQLITE_DONE) {
            // sqlite3_step() has finished executing
            more_ = false; 
        }
        else{
            STENCILA_SQLITE_THROW(db_,code);
        }
    }

    //! @brief
    //! @return
    unsigned int columns(void){
        begin();
        return sqlite3_column_count(stmt_);
    }

    //! @brief
    //! @param column
    //! @return
    std::string name(unsigned int column){
        begin();
        return sqlite3_column_name(stmt_,column);
    }

    //! @brief
    //! @return
    std::vector<std::string> names(void) {
        std::vector<std::string> result;
        for(unsigned int i=0;i<columns();i++) result.push_back(name(i));
        return result;
    }

    //! @brief
    //! @param column
    //! @return
    const Datatype& type(unsigned int column){
        begin();
        switch(sqlite3_column_type(stmt_,column)){
            case SQLITE_NULL:
                return Null;
            break;
            case SQLITE_INTEGER:
                return Integer;
            break;
            case SQLITE_FLOAT:
                return Real;
            break;
            case SQLITE_TEXT:
                return Text;
            break;
        }
        throw Exception("Undefined column type");
    }

    //! @brief
    //! @return
    std::vector<Datatype> types(void) {
        std::vector<Datatype> result;
        for(unsigned int i=0;i<columns();i++) result.push_back(type(i));
        return result;
    }

    //! @brief
    //! @param column
    //! @return
    template<typename Type>
    Type get(unsigned int column);
    
    template<
        typename Type = std::vector<std::string>,
        typename... Parameters
    >
    std::vector<Type> fetch(const Parameters&... pars) {
        std::vector<Type> rows;
        prepare();
        use(pars...);
        begin();
        while(more()) {
            Type row;
            for(unsigned int col=0;col<columns();col++) row.push_back(get<std::string>(col));
            rows.push_back(row);
            next();
        }
        return rows;
    }

    //! @brief
    //! @param pars
    //! @return
    template<
        typename Type = std::string,
        typename... Parameters
    >
    Type value(const Parameters&... pars) {
        prepare();
        use(pars...);
        begin();
        if(more()) return get<Type>(0);
        else throw Exception("No rows selected");
    }

    //! @brief
    //! @param pars
    //! @return
    template<
        typename Type = std::string,
        typename... Parameters
    >
    std::vector<Type> column(const Parameters&... pars) {
        std::vector<Type> column;
        prepare();
        use(pars...);
        begin();
        while(more()){
            column.push_back(get<Type>(0));
            next();
        }
        return column;
    }

    //! @brief
    //! @param pars
    //! @return
    template<
        typename Type = std::vector<std::string>,
        typename... Parameters
    >
    Type row(const Parameters&... pars) {
        Type row;
        prepare();
        use(pars...);
        begin();
        if(more()){
            for(unsigned int col=0;col<columns();col++) row.push_back(get<std::string>(col));
        }
        return row;
    }

};

template<>
inline
bool Cursor::get<bool>(unsigned int column){
    return sqlite3_column_int(stmt_, column);
}

template<>
inline
int Cursor::get<int>(unsigned int column){
    return sqlite3_column_int(stmt_, column);
}

template<>
inline
float Cursor::get<float>(unsigned int column){
    return sqlite3_column_double(stmt_, column);
}

template<>
inline
double Cursor::get<double>(unsigned int column){
    return sqlite3_column_double(stmt_, column);
}

template<>
inline
std::string Cursor::get<std::string>(unsigned int column){
    return reinterpret_cast<const char *>(sqlite3_column_text(stmt_, column));
}

}
}