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
//! @brief Definition of class Datatable

#pragma once

#include <string>
#include <vector>
#include <fstream>

#include <boost/filesystem.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/format.hpp>
#include <boost/tokenizer.hpp>
#include <boost/lexical_cast.hpp>

#include "exception.hpp"
#include "dataset.hpp"
#include "dataquery.hpp"

namespace Stencila {
	
//! @class Datatable
//! @brief A table of data in a dataset
class Datatable {
	
private:
	
	//! @brief Name of the Datatable
	std::string name_;

	//! @brief Whether the Datatable is contained in a Dataset (true) or maintains its own Dataset (false)
	bool contained_;
	
	//! @brief Dataset where the Datatable resides
	Dataset* dataset_;

public:
	
	//! @name Constructor & destructors
	//! @brief Create and destroy a Datatable
	//! @{

	//! @brief Create a Datatable object
	Datatable(const std::string& name="unnamed"):
		name_(name),
		contained_(false),
		dataset_(new Dataset()){
			
		execute("CREATE TABLE \""+name_+"\"(id INTEGER)");
	}

	//! @brief Create a Datatable object from an existing table in a Dataset
	//! @param dataset Dataset where this Datatable resides
	//! @param name Name of the table. This must be an existing database table.
	Datatable(const std::string& name, Dataset* dataset):
		name_(name),
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

	
	std::string name(void) const {
		return name_;
	}
	Datatable& name(const std::string& value) {
		//! @todo Catch an attempt to set an invalid name
		execute("ALTER TABLE \""+name()+"\" RENAME TO \""+value+"\"");
		name_ = value;
		return *this;
	}
	
	bool contained(void) const {
		return contained_;
	}
	
	Dataset& dataset(void) const {
		return *dataset_;
	}
	
	//! @brief Get the number of rows in the datatable
	//! @return Number of rows
	unsigned int rows(void) {
		return dataset().value<int>("SELECT count(*) FROM "+name());
	}
	
	//! @brief Get the number of columns in the datatable
	//! @return Number of columns
	unsigned int columns(void) {
		return dataset().cursor("SELECT * FROM "+name()).columns();
	}
	
	//! @brief Get the dimensions(rows x columns) of the datatable
	//! @return A vector with first item the number of rows and the second item the number of columns
	std::vector<unsigned int> dimensions(void) {
		return {rows(),columns()};
	}
	
	Datacolumn add(const std::string& column, const Datatype& type){
		//! @todo Add checking of type
		execute("ALTER TABLE \""+name()+"\" ADD COLUMN \""+column+"\" "+type.sql());
		return Datacolumn(column,this);
	}

	template<typename... Args>
	Datatable& add(const std::string& column, const Datatype& type, Args... args){
		add(column,type);
		add(args...);
		return *this;
	}
	
	//! @brief Get the name of a column in a datatable
	//! @param column The column index
	//! @return Column name
	std::string name(unsigned int column) {
		return dataset().cursor("SELECT * FROM "+name()).name(column);
	}
	
	//! @brief Get the names of all columns in the datatable
	//! @return Vector of column names
	std::vector<std::string> names(void) {
		return dataset().cursor("SELECT * FROM "+name()).names();
	}
	
	//! @brief Get the type name of a column in a datatable
	//! @param column The column index
	//! @return Column type
	Datatype type(unsigned int column) {
		return dataset().cursor("SELECT * FROM "+name()).type(column);
	}
	
	std::vector<Datatype> types(void) {
		return dataset().cursor("SELECT * FROM "+name()).types();
	}
	
	std::vector<std::string> indices(void) {
		return dataset().indices(name());
	}
	
	//! @}
	
	
	Datatable& save(const std::string& path=""){
		if(not contained()) dataset().save(path);
		else throw Exception("TODO: Extract this table to a separate file");
		return *this;
	}
	
	//! @name Data import/export methods
	//! @brief Methods for importing or exporting data to/from the Datatable
	//! @{
	
	//! @brief 
	//! @param path Path of the file to load
	//! @param header Whether or not the file has an initial header line of column names
	//! @return This Datatable
	Datatable& load(const std::string& path, const bool& header=true){
		// Check the file at path exists
		std::ifstream file(path);
		if(not file.is_open()) throw Exception("Unable to open file \""+path+"\"");
		
		std::string line;
		unsigned int count = 0;
		
		// Determine the type of file, CSV, TSV, fixed-width etc
		enum {csv,tsv} filetype;
		//! @todo Allow for more file types
		//! @todo Sniff the file to see if the filetype can be verified
		std::string extension = boost::filesystem::path(path).extension().string();
		if(extension==".csv") filetype = csv;
		else if(extension==".tsv") filetype = tsv;
		else throw Exception("Unrecognised file type");
		
		// Create a separator
		boost::escaped_list_separator<char> separator;
		if(filetype==csv) separator = boost::escaped_list_separator<char>('\\', ',', '\"');
		else if(filetype==tsv) separator = boost::escaped_list_separator<char>('\\', '\t', '\"');
		typedef boost::tokenizer<boost::escaped_list_separator<char> > Tokenizer;
		
		// Create column names
		std::vector<std::string> names;
		if(header){
			std::getline(file,line);
			Tokenizer tokenizer(line,separator);
			for(auto i=tokenizer.begin();i!=tokenizer.end();++i) names.push_back(*i);
		} else {
			//Detemine the number of columns in the first line
			std::getline(file,line);
			Tokenizer tokenizer(line,separator);
			//Create column names
			int count = 1;
			for(auto i=tokenizer.begin();i!=tokenizer.end();++i) {
				names.push_back("_"+boost::lexical_cast<std::string>(count));
			}
			//Go back to start of the file
			file.seekg(0);
		}
		
		// Create column types by reading a number of rows and attempting to convert 
		// to different types
		//! @todo Determine field types
		//! @todo Finish this off. Should read in the first 1000 rows into a vector so that these can be used below when doing actual inserts
		std::vector<std::string> types(names.size());
		auto position = file.tellg();
		count = 0;
		while(file.good() and count<100){
			std::getline(file,line);
			count++;
			boost::trim(line);
			if(line.length()==0) break;
			
			for(unsigned int i=0;i<names.size();i++){
				std::string value = "";
				try{
					boost::lexical_cast<double>(value);
					//successes[column][0]++;
				}
				catch(boost::bad_lexical_cast){
					try{
						boost::lexical_cast<int>(value);
						//successes[column][0]++;
					}
					catch(boost::bad_lexical_cast){
						
					}
				}
				types[i] = "TEXT";
			}
		}
		//Go back to start of data
		file.seekg(position);
		
		// Create temporary table
		std::string temp_name = "stencila_"+name()+"_temp";
		std::string create = "CREATE TABLE \""+temp_name+"\" (";
		for(unsigned int i=0;i<names.size();i++) {
			create += names[i] + " " + types[i];
			if(i!=names.size()-1) create += ",";
		}
		create += ")";
		execute(create);
		
		// Prepare an insert statement
		std::string insert = "INSERT INTO \""+temp_name+"\" VALUES (?";
		for(unsigned int i=1;i<names.size();i++) insert += ",?";
		insert += ")";
		Datacursor insert_cursor = cursor(insert);
		insert_cursor.prepare();
		
		count = 0;
		while(file.good()){
			
			std::getline(file,line);
			count++;
			boost::trim(line);
			if(line.length()==0) break;
			
			std::vector<std::string> row;
			boost::tokenizer<boost::escaped_list_separator<char> > tokenizer(line,separator);
			for(auto i=tokenizer.begin();i!=tokenizer.end();++i){
				row.push_back(*i);
			}
			
			//Check that row is the correct size
			if(row.size()!=names.size()) 
				throw Exception(boost::str(boost::format("Line %i has %i items but expected %i items")%count%row.size()%names.size()));
			
			for(unsigned int i=0;i<row.size();i++){
				insert_cursor.bind(i,row[i]);
			}
			
			insert_cursor.execute();
			insert_cursor.reset();
		}
		file.close();
		
		//! Replace the existing table with the new one
		execute("DROP TABLE IF EXISTS \""+name()+"\"");
		execute("ALTER TABLE \"stencila_"+name()+"_temp\" RENAME TO \""+name()+"\"");
		
		return *this;
	}
	
	//! @brief 
	//! @param filename Path of the file to create
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
	Datatable& execute(const std::string& sql){
		dataset().execute(sql);
		return *this;
	}
	
	//!
	Datacursor cursor(const std::string& sql){
		return dataset().cursor(sql);
	}
	
	template<typename Type = std::vector<std::string>>
	std::vector<Type> fetch(std::string sql) {
		return dataset().fetch<Type>(sql);
	}
	
	template<typename Type = std::string>
	std::vector<Type> column(std::string column){
		return dataset().column<Type>("SELECT \""+column+"\" FROM \""+name()+"\"");
	}
	
	template<typename Type = std::vector<std::string>>
	Type row(unsigned int row) {
		return dataset().row<Type>("SELECT * FROM \""+name()+"\" LIMIT 1 OFFSET "+ boost::lexical_cast<std::string>(row));
	}
	
	//! @}
	
	template<typename Type = std::vector<std::string>>
	std::vector<Type> fetch() {
		return dataset().fetch<Type>("SELECT * FROM \""+name()+"\"");
	}
	
	Datatable operator[](Dataquery dataquery) {
		dataquery.from(name());
		std::string sql = dataquery.sql();
		return dataset().select(sql);
	}
	
};

template<>
void print_format(const Datatable& datatable){
	print_object("Datatable",datatable,
		"name",datatable.name(),
		"contained",datatable.contained(),
		"dataset",datatable.dataset()
	);
}

}