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

//! @file dataset.hpp
//! @brief Definition of class Dataset

#pragma once

#include <string>
#include <sstream>
#include <vector>

#include <boost/foreach.hpp>
#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

#include "sqlite.hpp"
#include "datacursor.hpp"
#include "dataset-math-functions.hpp"
#include "dataset-math-aggregators.hpp"
#include "exception.hpp"
#include "hashing.hpp"

namespace Stencila {
	
class Datatable;
	
//! @class Dataset
//! @brief A set of related data
//! 
//! Datasets are a collection of related data residing in one or more Datatables.
//! A Dataset is just a database but has some additional features which make them easier to work with.
//! Currently SQLite is used as the Dataset database engine. Additional database engines may be added later.
class Dataset {
	
private:
	//! Unique resource identifier (URI) for this Dataset
	std::string uri_;

	//! SQLite database engine connection
	sqlite3* db_;
	
public:
	
	//! @name Constructors & destructors
	//! @brief Create and destroy a Dataset
	//! @{

	//! @brief Create a Dataset by optionally passing its URI
	//! @param uri The URI of the dataset. Currently can only be a local filename or ":memory:"
	Dataset(std::string uri = ""):
		uri_(uri){
		
		if(uri.length()==0) uri_ = ":memory:";
			
		int code_open = sqlite3_open(uri_.c_str(), &db_);
		if(code_open!=SQLITE_OK) throw Exception("sqlite3_open ("+uri_+") failed : "+sqlite3_errmsg(db_));
			
		//Create special Stencila tables and associated indices
		execute(
			"CREATE TABLE IF NOT EXISTS stencila_datatables ("
				"name TEXT,"
				"source INTEGER,"
				"sql TEXT,"
				"signature INTEGER,"
				"status INTEGER"
			");"
			"CREATE INDEX IF NOT EXISTS stencila_datatables_name ON stencila_datatables(name);"
			"CREATE INDEX IF NOT EXISTS stencila_datatables_signature ON stencila_datatables(signature);"
			"CREATE INDEX IF NOT EXISTS stencila_datatables_status ON stencila_datatables(status);"
		);
		
		//Register functions
		MathFunctions::create(db_);
        MathAggregators::create(db_);
	}
	
	//! @brief Destroys the memory held by the Dataset
	~Dataset(void){
		if(db_) sqlite3_close(db_);
	}
	
	//! @}

	//! @name Attribute methods
	//! @brief Get attributes of the Dataset
	//! @{
	
	//! @brief Get the URI of the Dataset.
	//! @return A URI
	std::string uri(void) const {
		return uri_;
	}
	
	//! @brief Get a list of the Datatables in the Dataset.
	//! @return A vector of names of tables
	std::vector<std::string> tables(void) {
		return column("SELECT name FROM sqlite_master WHERE type=='table' AND name NOT LIKE 'stencila_%'");
	}
		
	//! @brief Get a list of the indices in the entire Dataset or for a particular table.
	//! @param table The name of the table for which the lists of indices is wanted
	//! @return A vector of names of indices
	std::vector<std::string> indices(const std::string& table="") {
		std::string sql = "SELECT name FROM sqlite_master WHERE type=='index' AND name NOT LIKE 'stencila_%'";
		if(table.length()==0) return column(sql);
		else  return column(sql+"AND tbl_name=='"+table+"'");
	}
	
	//! @}
	
	
	//! @name Saving methods
	//! @brief Save a Dataset
	//! @{
	
	//! @brief Save the dataset to a local file
	//! @param uri The URI filename to save the Dataset to.
	Dataset& save(const std::string& uri="",bool backup=false){
		
		//Make any cached query tables permanent
		BOOST_FOREACH(std::string table,column("SELECT name FROM stencila_datatables WHERE status==0")){
			execute("CREATE TABLE "+table+" AS SELECT * FROM "+table);
			execute("UPDATE stencila_datatables SET status=1 WHERE name=='"+table+"'");
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
	
	Dataset& backup(const std::string& path){
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
			return value<int>("SELECT count(*) FROM stencila_datatables WHERE status<2"); 
		}
		else {
			std::string signature = boost::lexical_cast<std::string>(Hash(sql));
			return value<int>("SELECT count(*) FROM stencila_datatables WHERE signature=="+signature);
		}
	}
	
	Dataset& vacuum(void) {
		BOOST_FOREACH(std::string name,column("SELECT name FROM stencila_datatables WHERE status<2")){
			execute("DROP TABLE "+name);
			execute("DELETE FROM stencila_datatables WHERE name==?",name);
		}
		execute("VACUUM");
		return *this;
	}	
	
	//! @}
	
	
	//! @name SQL execution and query methods
	//! @brief Execute SQL or query the Dataset to extract data
	//! @{
	
	//! @brief Get a Datacursor for an SQL statement to be executed on this Dataset
	//! @param sql An SQL statement
	//! @return A Datacursor
	//! @see Dataset::execute
	Datacursor cursor(const std::string& sql) {
		return Datacursor(db_,sql);
	}
	
	//! @brief Execute SQL on the Dataset
	//! @param sql A SQL statement
	//! @return This Dataset
	//Dataset& execute(const std::string& sql) {
	//	cursor(sql).execute();
	//	return *this;
	//}
	
	template<typename... Parameters>
	Dataset& execute(const std::string& sql, const Parameters&... pars) {
		cursor(sql).execute(pars...);
		return *this;
	}
	
	//! @brief Execute a SQL SELECT statement on the Dataset and return a vector of rows
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

	//! @brief Execute a SQL SELECT statement on the Dataset and return a single value.
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
	
	//! @brief Execute a SQL SELECT statement on the Dataset and return the first column.
	//! @see Datacursor::column
	//! @param sql A SQL SELECT statement
	//! @return A vector representing the column
	template<
		typename Type = std::string,
		typename... Parameters		
	>
	std::vector<Type> column(const std::string& sql, const Parameters&... pars) {
		return cursor(sql).column<Type>(pars...);
	}
	
	//! @brief Execute a SQL SELECT statement on the Dataset and return the first row.
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
	
	//! @brief Import a database table to a Datatable
	//! @param name The name of the table
	//! @return A Datatable
	Datatable import(const std::string& name);
	
	//! @brief Create a Datatable in the Dataset
	//! @param name The name of the table
	//! @return The new Datatable
	// Needs to be defined in datatable.hpp
	template<typename... Args>
	Datatable create(const std::string& name, Args... args);
	
	//! @brief Get a Datatable in the Dataset
	//! @param name The name of the table
	//! @return A Datatable
	// Needs to be defined in datatable.hpp
	Datatable table(const std::string& name);
	
	Datatable select(const std::string& sql);
	
};
	
} 