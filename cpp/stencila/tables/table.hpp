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

//! @file table.hpp
//! @brief Definition of Table class
//! @author Nokome Bentley

#pragma once

#include <stencila/tables/tableset.hpp>

namespace Stencila {
namespace Tables {

//! @class Table
//! @brief A table of data in a Tableset
class Table {

private:

    //! @brief Name of the Table
    std::string name_;
    
    //! @brief Whether the Table has been created in the Tableset yet
    bool created_;

    //! @brief Whether the Table is contained in a Tableset (true) or maintains its own Tableset (false)
    bool contained_;

    //! @brief Tableset where the Table resides
    Tableset* tableset_;

public:
    
    //! @name Constructor & destructors
    //! @brief Create and destroy a Table
    //! @{

    //! @brief Create a Table object
    Table(void):
        name_("stencila_"+boost::lexical_cast<std::string>(Hash())),
        created_(false),
        contained_(false),
        tableset_(new Tableset()){
    }

    //! @brief Create a Table object
    template<typename... Columns>
    Table(const std::string& name,Columns... columns):
        name_(name),
        created_(true),
        contained_(false),
        tableset_(new Tableset()){
        tableset().create(name,columns...);
    }

    //! @brief Create a Table object from an existing table in a Tableset
    //! @param tableset Tableset where this Table resides
    //! @param name Name of the table. This must be an existing database table.
    Table(const std::string& name, Tableset* tableset, bool created = true):
        name_(name),
        created_(created),
        contained_(true),
        tableset_(tableset){
    }
    
    //! @brief Destroys the memory held by the Table
    ~Table(void){
        if(not contained_) delete tableset_;
    }
    
    //! @}
    
    //! @name Attribute methods
    //! @brief Get attributes of the table
    //! @{

    //! @brief
    //! @return
    std::string name(void) const {
        return name_;
    }

    //! @brief
    //! @param value
    //! @return
    Table& name(const std::string& value) {
        tableset().rename(name(),value);
        name_ = value;
        return *this;
    }

    //! @brief 
    //! @return Whether the table has been created or not
    bool created(void) const {
        return created_;
    }
    
    void modified(void) const {
        return tableset().modified(name());
    }

    //! @brief
    //! @return
    bool contained(void) const {
        return contained_;
    }

    //! @brief
    //! @return
    Tableset& tableset(void) const {
        return *tableset_;
    }
    
    //! @brief Get the number of rows in the table
    //! @return Number of rows
    unsigned int rows(void) const {
        return created_?(tableset().value<int>("SELECT count(*) FROM "+name())):0;
    }
    
    //! @brief Get the number of columns in the table
    //! @return Number of columns
    unsigned int columns(void) const {
        return created_?(tableset().cursor("SELECT * FROM "+name()).columns()):0;
    }
    
    //! @brief Get the dimensions(rows x columns) of the table
    //! @return A vector with first item the number of rows and the second item the number of columns
    std::vector<unsigned int> dimensions(void) const {
        return {rows(),columns()};
    }

    //! @brief
    //! @param column_name
    //! @param type
    //! @param args
    //! @return
    template<typename... Args>
    Table& add(const std::string& column_name, const Datatype& type, Args... args){
        //! @todo Add checking of type
        if(created_) {
            execute("ALTER TABLE \""+name()+"\" ADD COLUMN \""+column_name+"\" "+type.sql());
        }
        else {
            execute("CREATE TABLE \""+name()+"\" (\""+column_name+"\" "+type.sql()+")");
            created_ = true;
        }
        add(args...);
        return *this;
    }

    //! @brief
    //! @return
    Table& add(void){
        return *this;
    }
    
    //! @brief Get the name of a column in a table
    //! @param column The column index
    //! @return Column name
    std::string name(unsigned int column) const{
        return tableset().cursor("SELECT * FROM \""+name()+"\"").name(column);
    }
    
    //! @brief Get the names of all columns in the table
    //! @return Vector of column names
    std::vector<std::string> names(void)  const{
        return created_?(tableset().cursor("SELECT * FROM \""+name()+"\"").names()):(std::vector<std::string>{});
    }
    
    //! @brief Get the type name of a column in a table
    //! @param column The column index
    //! @return Column type
    Datatype type(unsigned int column) const {
        return tableset().cursor("SELECT * FROM \""+name()+"\"").type(column);
    }

    //! @brief
    //! @return
    std::vector<Datatype> types(void) const {
        return created_?(tableset().cursor("SELECT * FROM \""+name()+"\"").types()):(std::vector<Datatype>{});
    }

    //! @brief
    //! @param columns
    //! @return
    template<typename... Columns>
    void index(Columns... columns) const {
        tableset().index(name(),columns...);
    }

    //! @brief
    //! @param columns
    //! @return
    void index(const std::vector<std::string>& columns) const {
        tableset().index(name(),columns);
    }

    //! @brief
    //! @return
    std::vector<std::string> indices(void) const {
        return tableset().indices(name());
    }

    //! @}

    //! @brief
    //! @param path
    //! @return
    Table& save(const std::string& path="") {
        if(not contained()) tableset().save(path);
        else throw Exception("TODO: Extract this table to a separate file");
        return *this;
    }

    //! @brief
    //! @param columns
    //! @return    
    std::string append_sql(unsigned int columns){
        // Prepare an insert statement with bindings
        std::string sql = "INSERT INTO \""+name()+"\" VALUES (?";
        for(unsigned int i=1;i<columns;i++) sql += ",?";
        sql += ")";
        return sql;
    }

    //! @brief
    //! @param row
    //! @return
    template<typename Container = std::vector<std::string>>
    Table& append(const Container& row){
        Cursor insert_cursor = cursor(append_sql(row.size()));
        insert_cursor.prepare();
        for(unsigned int i=0;i<row.size();i++){
            //SQLite uses 1-based indexing for statement parameters
            insert_cursor.bind(i+1,row[i]);
        }
        insert_cursor.execute();
        return *this;
    }

    //! @brief
    //! @param table
    //! @return
    const Table& append(const Table& table) const {
        execute("INSERT INTO \"" + name() + "\" SELECT * FROM \"" + table.name() + "\"");
        return *this;
    }
    
    //! @name Data import/export methods
    //! @brief Methods for importing or exporting data to/from the Table
    //! @{
    
    //! @brief 
    //! @param path Path of the file to load
    //! @param header Whether or not the file has an initial header line of column names
    //! @return This Table
    Table& load(const std::string& path, bool header=true){
        tableset().load(name(),path,header);
        return *this;
    }
    
    //! @brief 
    //! @param path Path of the file to create
    //! @return This Table
    Table& dump(const std::string& path){
        return *this;
    }
    
    //! @}
    
    //! @name SQL methods
    //! @brief Convienience methods for executing SQL on tableset
    //! @{
    
    //! @brief Execute SQL but do not return anything. Used for UPDATE, INSERT etc SQL statements
    //! @param sql An SQL string
    //! @return This table
    const Table& execute(const std::string& sql) const {
        tableset().execute(sql);
        return *this;
    }

    //! @brief
    //! @param sql
    //! @return
    Cursor cursor(const std::string& sql) const {
        return tableset().cursor(sql);
    }

    //! @brief
    //! @param sql
    //! @return
    template<typename Type = std::vector<std::string>>
    std::vector<Type> fetch(std::string sql) const {
        return tableset().fetch<Type>(sql);
    }

    //! @brief
    //! @param col
    //! @return
     template<typename Type = std::string>
    Type value(unsigned int row, unsigned int col) const {
        return tableset().value<Type>("SELECT \""+name(col)+"\" FROM \""+name()+"\" LIMIT 1 OFFSET " + boost::lexical_cast<std::string>(row));
    }

    //! @brief
    //! @param where
    //! @return    
    template<typename Type = std::string>
    Type value(const std::string& columns,const std::string& where="1") const {
        return tableset().value<Type>("SELECT "+columns+" FROM \""+name()+"\" WHERE "+where+" LIMIT 1;");
    }

    //! @brief
    //! @param column
    //! @return 
    template<typename Type = std::string>
    std::vector<Type> column(std::string column) const {
        return tableset().column<Type>("SELECT \""+column+"\" FROM \""+name()+"\"");
    }

    //! @brief
    //! @param row
    //! @return
    template<typename Type = std::vector<std::string>>
    Type row(unsigned int row) const {
        return tableset().row<Type>("SELECT * FROM \""+name()+"\" LIMIT 1 OFFSET "+ boost::lexical_cast<std::string>(row));
    }
    
    //! @}
    
    template<typename Type = std::vector<std::string>>
    std::vector<Type> fetch() const {
        return tableset().fetch<Type>("SELECT * FROM \""+name()+"\"");
    }

    //! @brief
    //! @param sql
    //! @param reuse
    //! @param cache
    //! @return
    Table select(const std::string& sql, bool reuse = true) const {
        return tableset().select(sql,reuse);
    }

    //! @brief
    //! @param rows
    //! @return
    Table head(const unsigned int rows = 10) const {
        return tableset().select("SELECT * FROM \""+name()+"\" LIMIT "+boost::lexical_cast<std::string>(rows));
    }
    
    //! @brief
    //! @param rows
    //! @return
    Table tail(const unsigned int rows = 10) const {
        return tableset().select("SELECT * FROM \""+name()+"\" ORDER BY rowid DESC LIMIT "+boost::lexical_cast<std::string>(rows));
    }

    //! @brief
    //! return
    Table clone(void) const {
        return tableset().clone(name());
    }
};

template<typename... Columns>
Table Tableset::create(const std::string& name, Columns... columns){
    std::string sql = "CREATE TABLE " + name + "(" + Tableset_create_helper(columns...) + ");";
    execute(sql);
    return table(name);
}

}
}