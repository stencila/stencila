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

//! @file datatable.hpp
//! @brief Definition of Datatable class
//! @author Nokome Bentley

#pragma once

#include <stencila/dataset.hpp>

namespace Stencila {

//! @class Datatable
//! @brief A table of data in a dataset
class Datatable {

private:

    //! @brief Name of the Datatable
    std::string name_;
    
    //! @brief Whether the Datatable has been created in the Dataset yet
    bool created_;

    //! @brief Whether the Datatable is contained in a Dataset (true) or maintains its own Dataset (false)
    bool contained_;

    //! @brief Dataset where the Datatable resides
    Dataset* dataset_;

public:
    
    //! @name Constructor & destructors
    //! @brief Create and destroy a Datatable
    //! @{

    //! @brief Create a Datatable object
    Datatable(void):
        name_("stencila_"+boost::lexical_cast<std::string>(Hash())),
        created_(false),
        contained_(false),
        dataset_(new Dataset()){
    }

    //! @brief Create a Datatable object
    template<typename... Columns>
    Datatable(const std::string& name,Columns... columns):
        name_(name),
        created_(true),
        contained_(false),
        dataset_(new Dataset()){
        dataset().create(name,columns...);
    }

    //! @brief Create a Datatable object from an existing table in a Dataset
    //! @param dataset Dataset where this Datatable resides
    //! @param name Name of the table. This must be an existing database table.
    Datatable(const std::string& name, Dataset* dataset, bool created = true):
        name_(name),
        created_(created),
        contained_(true),
        dataset_(dataset){
    }
    
    //! @brief Destroys the memory held by the Datatable
    ~Datatable(void){
        if(not contained_) delete dataset_;
    }
    
    //! @}
    
    //! @name Attribute methods
    //! @brief Get attributes of the datatable
    //! @{

    //! @brief
    //! @return
    std::string name(void) const {
        return name_;
    }

    //! @brief
    //! @param value
    //! @return
    Datatable& name(const std::string& value) {
        dataset().rename(name(),value);
        name_ = value;
        return *this;
    }

    //! @brief 
    //! @return Whether the datatable has been created or not
    bool created(void) const {
        return created_;
    }
    
    void modified(void) const {
        return dataset().modified(name());
    }

    //! @brief
    //! @return
    bool contained(void) const {
        return contained_;
    }

    //! @brief
    //! @return
    Dataset& dataset(void) const {
        return *dataset_;
    }
    
    //! @brief Get the number of rows in the datatable
    //! @return Number of rows
    unsigned int rows(void) const {
        return created_?(dataset().value<int>("SELECT count(*) FROM "+name())):0;
    }
    
    //! @brief Get the number of columns in the datatable
    //! @return Number of columns
    unsigned int columns(void) const {
        return created_?(dataset().cursor("SELECT * FROM "+name()).columns()):0;
    }
    
    //! @brief Get the dimensions(rows x columns) of the datatable
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
    Datatable& add(const std::string& column_name, const Datatype& type, Args... args){
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
    Datatable& add(void){
        return *this;
    }
    
    //! @brief Get the name of a column in a datatable
    //! @param column The column index
    //! @return Column name
    std::string name(unsigned int column) const{
        return dataset().cursor("SELECT * FROM \""+name()+"\"").name(column);
    }
    
    //! @brief Get the names of all columns in the datatable
    //! @return Vector of column names
    std::vector<std::string> names(void)  const{
        return created_?(dataset().cursor("SELECT * FROM \""+name()+"\"").names()):(std::vector<std::string>{});
    }
    
    //! @brief Get the type name of a column in a datatable
    //! @param column The column index
    //! @return Column type
    Datatype type(unsigned int column) const {
        return dataset().cursor("SELECT * FROM \""+name()+"\"").type(column);
    }

    //! @brief
    //! @return
    std::vector<Datatype> types(void) const {
        return created_?(dataset().cursor("SELECT * FROM \""+name()+"\"").types()):(std::vector<Datatype>{});
    }

    //! @brief
    //! @param columns
    //! @return
    template<typename... Columns>
    void index(Columns... columns) const {
        dataset().index(name(),columns...);
    }

    //! @brief
    //! @param columns
    //! @return
    void index(const std::vector<std::string>& columns) const {
        dataset().index(name(),columns);
    }

    //! @brief
    //! @return
    std::vector<std::string> indices(void) const {
        return dataset().indices(name());
    }

    //! @}

    //! @brief
    //! @param path
    //! @return
    Datatable& save(const std::string& path="") {
        if(not contained()) dataset().save(path);
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
    Datatable& append(const Container& row){
        Datacursor insert_cursor = cursor(append_sql(row.size()));
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
    const Datatable& append(const Datatable& table) const {
        execute("INSERT INTO \"" + name() + "\" SELECT * FROM \"" + table.name() + "\"");
        return *this;
    }
    
    //! @name Data import/export methods
    //! @brief Methods for importing or exporting data to/from the Datatable
    //! @{
    
    //! @brief 
    //! @param path Path of the file to load
    //! @param header Whether or not the file has an initial header line of column names
    //! @return This Datatable
    Datatable& load(const std::string& path, bool header=true){
        dataset().load(name(),path,header);
        return *this;
    }
    
    //! @brief 
    //! @param path Path of the file to create
    //! @return This Datatable
    Datatable& dump(const std::string& path){
        return *this;
    }
    
    //! @}
    
    //! @name SQL methods
    //! @brief Convienience methods for executing SQL on dataset
    //! @{
    
    //! @brief Execute SQL but do not return anything. Used for UPDATE, INSERT etc SQL statements
    //! @param sql An SQL string
    //! @return This datatable
    const Datatable& execute(const std::string& sql) const {
        dataset().execute(sql);
        return *this;
    }

    //! @brief
    //! @param sql
    //! @return
    Datacursor cursor(const std::string& sql) const {
        return dataset().cursor(sql);
    }

    //! @brief
    //! @param sql
    //! @return
    template<typename Type = std::vector<std::string>>
    std::vector<Type> fetch(std::string sql) const {
        return dataset().fetch<Type>(sql);
    }

    //! @brief
    //! @param col
    //! @return
     template<typename Type = std::string>
    Type value(unsigned int row, unsigned int col) const {
        return dataset().value<Type>("SELECT \""+name(col)+"\" FROM \""+name()+"\" LIMIT 1 OFFSET " + boost::lexical_cast<std::string>(row));
    }

    //! @brief
    //! @param where
    //! @return    
    template<typename Type = std::string>
    Type value(const std::string& columns,const std::string& where="1") const {
        return dataset().value<Type>("SELECT "+columns+" FROM \""+name()+"\" WHERE "+where+" LIMIT 1;");
    }

    //! @brief
    //! @param column
    //! @return 
    template<typename Type = std::string>
    std::vector<Type> column(std::string column) const {
        return dataset().column<Type>("SELECT \""+column+"\" FROM \""+name()+"\"");
    }

    //! @brief
    //! @param row
    //! @return
    template<typename Type = std::vector<std::string>>
    Type row(unsigned int row) const {
        return dataset().row<Type>("SELECT * FROM \""+name()+"\" LIMIT 1 OFFSET "+ boost::lexical_cast<std::string>(row));
    }
    
    //! @}
    
    template<typename Type = std::vector<std::string>>
    std::vector<Type> fetch() const {
        return dataset().fetch<Type>("SELECT * FROM \""+name()+"\"");
    }

    //! @brief
    //! @param sql
    //! @param reuse
    //! @param cache
    //! @return
    Datatable select(const std::string& sql, bool reuse = true) const {
        return dataset().select(sql,reuse);
    }

    //! @brief
    //! @param rows
    //! @return
    Datatable head(const unsigned int rows = 10) const {
        return dataset().select("SELECT * FROM \""+name()+"\" LIMIT "+boost::lexical_cast<std::string>(rows));
    }
    
    //! @brief
    //! @param rows
    //! @return
    Datatable tail(const unsigned int rows = 10) const {
        return dataset().select("SELECT * FROM \""+name()+"\" ORDER BY rowid DESC LIMIT "+boost::lexical_cast<std::string>(rows));
    }

    //! @brief
    //! return
    Datatable clone(void) const {
        return dataset().clone(name());
    }
};

template<typename... Columns>
Datatable Dataset::create(const std::string& name, Columns... columns){
    std::string sql = "CREATE TABLE " + name + "(" + Dataset_create_helper(columns...) + ");";
    execute(sql);
    return table(name);
}

}
