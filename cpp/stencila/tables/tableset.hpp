/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file engine.hpp
//! @brief Definition of class Tableset::Tableset
//! @author Nokome Bentley

#pragma once

#include <string>
#include <sstream>
#include <fstream>
#include <vector>

#include <boost/foreach.hpp>
#include <boost/format.hpp>
#include <boost/tokenizer.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/join.hpp>
#include <boost/filesystem.hpp>

#include <stencila/exception.hpp>
#include <stencila/hashing.hpp>

#include <stencila/datatypes.hpp>

#include <stencila/tables/cursor.hpp>
#include <stencila/tables/functions.hpp>
#include <stencila/tables/aggregators.hpp>

namespace Stencila {
namespace Tables {

class Table;

//! @class Tableset
//! @brief A set of related tables
//! 
//! Tableset are a collection of related data residing in one or more Table.
//! A Tableset is just a database but has some additional features which make them easier to work with.
//! Currently SQLite is used as the Tableset database engine. Additional database engines may be added later.
class Tableset {

private:
     //! Unique resource identifier (URI) for this Tableset
     std::string uri_;

     //! SQLite database engine connection
     sqlite3* db_;

public:

    //! @name Constructors & destructors
    //! @brief Create and destroy a Tableset
    //! @{

    //! @brief Create a Tableset by optionally passing its URI
    //! @param uri The URI of the Tableset. Currently can only be a local filename or ":memory:"
    Tableset(std::string uri = ""):
        uri_(uri){

        if(uri.length()==0) uri_ = ":memory:";

        int code_open = sqlite3_open(uri_.c_str(), &db_);
        if(code_open!=SQLITE_OK) throw Exception("sqlite3_open ("+uri_+") failed : "+sqlite3_errmsg(db_));

        //Create special Stencila tables and associated indices
        execute(
        "CREATE TABLE IF NOT EXISTS stencila_tables ("
            "name TEXT,"
            "source INTEGER,"
            "sql TEXT,"
            "signature INTEGER,"
            "status INTEGER"
        ");"
        "CREATE INDEX IF NOT EXISTS stencila_tables_name ON stencila_tables(name);"
        "CREATE INDEX IF NOT EXISTS stencila_tables_signature ON stencila_tables(signature);"
        "CREATE INDEX IF NOT EXISTS stencila_tables_status ON stencila_tables(status);"
        );

        //Register functions
        Functions::create(db_);
        Aggregators::create(db_);
    }

    //! @brief Destroys the memory held by the Tableset
    ~Tableset(void){
        if(db_) sqlite3_close(db_);
    }

    //! @}


    //! @name Attribute methods
    //! @brief Get attributes of the Tableset
    //! @{

    //! @brief Get the URI of the Tableset.
    //! @return A URI
    std::string uri(void) const {
        return uri_;
    }

    //! @}

    //! @name Table methods
    //! @brief Create and get Tableset
    //! @{

    Table create(const std::string& name);

    //! @brief Create a Table in the Tableset
    //! @param name The name of the table
    //! @return The new Table
    template<typename... Args>
    Table create(const std::string& name, Args... args);

    //! @brief Import a database table to a Table
    //! @param name The name of the table
    //! @return A Table
    Table import(const std::string& name);

    //! @brief Load a file to a Table
    //! @param name Name of the new Table
    //! @param path Path of the file to be loaded
    //! @return A Table
    Table load(const std::string& name,const std::string& path,bool header=true);

    //! @brief Get a list of the Tableset in the Tableset.
    //! @return A vector of names of tables
    std::vector<std::string> tables(void) {
        return column("SELECT name FROM sqlite_master WHERE type=='table' AND name NOT LIKE 'stencila_%'");
    }

    //! @brief Get a Table in the Tableset
    //! @param name The name of the table
    //! @return A Table
    Table table(const std::string& name);

    //! @brief Rename a Table
    //! @param name The name of the table
    //! @return A Table
    //!
    //! This method is provided to encapsulate the implementation of caching within Tablesets
    //! Instead of calling this method directly you would normally call `Table::name()`
    Table rename(const std::string& name, const std::string& value);

    //! @brief Drop a Table
    //! @param name The name of the table
    void drop(const std::string& name){
        execute("DROP TABLE IF EXISTS \"" + name + "\"");
        execute("DELETE FROM stencila_tables WHERE name==\"" + name + "\"");
    }

    //! @}

    //! @name Index related methods
    //! @brief Create and get indices
    //! @{

private:

    //! @brief 
    //! @param column
    //! @return 
    std::string index_helper(const std::string& column){
        return column;
    }

    template<typename... Columns>
    std::string index_helper(const std::string& separator, const std::string& column, Columns... columns){
        return index_helper(column) + separator + index_helper(separator,columns...);
    }

public:

    template<typename... Columns>
    void index(const std::string& table, Columns... columns){
        std::string sql = "CREATE INDEX " + table + "_" + index_helper("_",columns...) + "_index ON " + table + "(" + index_helper(",",columns...) + ");";
        execute(sql);
    }

    //! @brief 
    //! @param table
    //! @param columns
    void index(const std::string& table, const std::vector<std::string>& columns){
        std::string sql = "CREATE INDEX " + table + "_" + boost::algorithm::join(columns, "_") + "_index ON " + table + "(" + boost::algorithm::join(columns, ",") + ");";
        execute(sql);
    }

    //! @brief Get a list of the indices in the entire Tableset or for a particular table.
    //! @param table The name of the table for which the lists of indices is wanted
    //! @return A vector of names of indices
    std::vector<std::string> indices(const std::string& table="") {
        std::string sql = "SELECT name FROM sqlite_master WHERE type=='index' AND name NOT LIKE 'stencila_%'";
        if(table.length()==0) return column(sql);
        else return column(sql+"AND tbl_name=='"+table+"'");
    }

    //! @}

    //! @name Saving methods
    //! @brief Save a Tableset
    //! @{

    //! @brief Save the Tableset to a local file
    //! @param uri The URI filename to save the Tableset to.
    Tableset& save(const std::string& uri="",bool backup=false){

        //Make any cached query tables permanent
        BOOST_FOREACH(std::string table,column("SELECT name FROM stencila_tables WHERE status==0")){
           execute("CREATE TABLE "+table+" AS SELECT * FROM "+table);
           execute("UPDATE stencila_tables SET status=1 WHERE name=='"+table+"'");
        }

        if(uri.length()>0 and uri!=uri_){
            sqlite3* to;
            int code_open = sqlite3_open(uri.c_str(), &to);
            if(code_open) throw Exception(("Unable to open : "+uri).c_str());

            // Uses sqlite3_backup functionality to copy database. See http://www.sqlite.org/backup.html
            sqlite3_backup* backup = sqlite3_backup_init(to, "main", db_, "main");
            if(backup){
                sqlite3_backup_step(backup,-1);
                sqlite3_backup_finish(backup);
            }

            //! @todo When closing connections check that db is not busy. See http://www.sqlite.org/capi3ref.html#sqlite3_close
            if(backup){
                //Close connection to copy
                if(sqlite3_close(to)!=SQLITE_OK) throw Exception(std::string("sqlite3_close failed : ")+sqlite3_errmsg(to));
            } else {
                //Close connection to old database
                if(sqlite3_close(db_)!=SQLITE_OK) throw Exception(std::string("sqlite3_close failed : ")+sqlite3_errmsg(db_));
                
                //Finally, make sure I point to the right places
                db_ = to;
                uri_ = uri;
            }
        }
        return *this;
    }

     //! @brief 
     //! @param path
     //! @return 
     Tableset& backup(const std::string& path){
        return save(path,true);
     }

     //! @}

     //! @name Cache related methods
     //! @brief Get information and manipulate the cache
     //! @{
     
     //! @brief Get the number of queries stored in the cache
     //! @return Number of queries
     int cached(const std::string& sql = "") {
          if(sql.length()==0){
                return value<int>("SELECT count(*) FROM stencila_tables WHERE status<2"); 
          }
          else {
               std::string signature = boost::lexical_cast<std::string>(Hash(sql));
               return value<int>("SELECT count(*) FROM stencila_tables WHERE signature=="+signature);
          }
     }

    //! @brief 
    //! @param table
     void modified(const std::string& table) {
          execute("UPDATE stencila_tables SET signature=NULL WHERE name==\"" + table + "\"");
     }
     
     Tableset& vacuum(void) {
          BOOST_FOREACH(std::string name,column("SELECT name FROM stencila_tables WHERE status<2")){
               execute("DROP TABLE "+name);
               execute("DELETE FROM stencila_tables WHERE name==?",name);
          }
          execute("VACUUM");
          return *this;
     }

    //! @}

    //! @name SQL execution and query methods
    //! @brief Execute SQL or query the Tableset to extract data
    //! @{

    //! @brief Get a Cursor for an SQL statement to be executed on this Tableset
    //! @param sql An SQL statement
    //! @return A Cursor
    //! @see Tableset::execute
    Cursor cursor(const std::string& sql) {
        return Cursor(db_,sql);
    }

    //! @brief Execute SQL on the Tableset
    //! @param sql A SQL statement
    //! @return This Tableset
    //Tableset& execute(const std::string& sql) {
    //             cursor(sql).execute();
    //             return *this;
    //}

    template<typename... Parameters>
        Tableset& execute(const std::string& sql, const Parameters&... pars) {
        cursor(sql).execute(pars...);
        return *this;
    }

    //! @brief Execute a SQL SELECT statement on the Tableset and return a vector of rows
    //! @see Datacursor::fetch
    //! @param sql A SQL SELECT statement
    //! @return A vector with each item containing a row
    template<
        typename Type = std::vector<std::string>,
        typename... Parameters
    >
    std::vector<Type> fetch(const std::string& sql, const Parameters&... pars) {
        return cursor(sql).fetch<Type>(pars...);
    }

    //! @brief Execute a SQL SELECT statement on the Tableset and return a single value.
    //! @see Datacursor::value
    //! @param sql A SQL SELECT statement
    //! @return A single value of type Type
    template<
        typename Type = std::string,
        typename... Parameters
    >
    Type value(const std::string& sql, const Parameters&... pars) {
        return cursor(sql).value<Type>(pars...);
    }

    //! @brief Execute a SQL SELECT statement on the Tableset and return the first column.
    //! @see Datacursor::column
    //! @param sql A SQL SELECT statement
    //! @return A vector representing the column
    template<
        typename Type = std::string,
        typename... Parameters
    >
    //! @brief 
    //! @param sql
    //! @param pars
    //! @return 
    std::vector<Type> column(const std::string& sql, const Parameters&... pars) {
        return cursor(sql).column<Type>(pars...);
    }

    //! @brief Execute a SQL SELECT statement on the Tableset and return the first row.
    //! @see Datacursor::row
    //! @param sql A SQL SELECT statement
    //! @return A vector representing the row
    template<
        typename Type = std::vector<std::string>,
        typename... Parameters
    >
    Type row(const std::string& sql, const Parameters&... pars) {
        return cursor(sql).row<Type>(pars...);
    }

    //! @}

    Table select(const std::string& sql, bool reuse = true);

    Table clone(const std::string& original);
};

inline
std::string Tableset_create_helper(void){
    return "";
}

inline
std::string Tableset_create_helper(const std::string& column, const Datatype& type){
    return column + " " + type.sql();
}

template<typename... Columns>
inline
std::string Tableset_create_helper(const std::string& column, const Datatype& type, Columns... columns){
    return Tableset_create_helper(column,type) + "," + Tableset_create_helper(columns...);
}

} 
}