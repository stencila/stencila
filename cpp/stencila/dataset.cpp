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

//! @file dataset.cpp
//! @brief Implmentations of Dataset methods which are unable to go into dataset.hpp because they return a Datatable
//! @author Nokome Bentley

#include <stencila/dataset.hpp>
#include <stencila/datatable.hpp>

namespace Stencila {

//! @brief 
//! @param name
//! @return 
Datatable Dataset::create(const std::string& name){
     return Datatable(name,this,false);
}

//! @brief 
//! @param name
//! @return 
Datatable Dataset::table(const std::string& name){
     return Datatable(name,this);
}
//! @brief 
//! @param name
//! @param value
//! @return 
Datatable Dataset::rename(const std::string& name, const std::string& value){
    //! @todo Catch an attempt to set an invalid name
    execute("ALTER TABLE \"" + name + "\" RENAME TO \"" + value + "\"");
    execute("UPDATE stencila_datatables SET name=\"" + value + "\" WHERE name==\"" + name + "\"");
    return table(name);
}

//! @brief 
//! @param name 
//! @param path
//! @param header
//! @return 
Datatable Dataset::load(const std::string& name, const std::string& path, bool header){
    // Check the file at path exists
    std::ifstream file(path);
    if(not file.is_open()) STENCILA_THROW(Exception,"Unable to open file \""+path+"\"");
    
    std::string line;
    unsigned int count = 0;
    
    // Determine the type of file, CSV, TSV, fixed-width etc
    enum {csv,tsv} filetype;
    //! @todo Allow for more file types
    //! @todo Sniff the file to see if the filetype can be verified
    std::string extension = boost::filesystem::path(path).extension().string();
    if(extension==".csv") filetype = csv;
    else if(extension==".tsv") filetype = tsv;
    else STENCILA_THROW(Exception,"Unrecognised file type");
    
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

    // Determine the type of each column by reading a number of rows and attempting to convert 
    //! @todo Perhaps this should read in the first 1000 rows into a vector so that these can be used below when doing actual inserts
    //! rather than re-reading the file.
    //! @todo Count the number of unique levels of integer and text columns and if under a cetain number then make columns nominal/ordinal.
    std::vector<int> flags(names.size(),0);
    auto position = file.tellg();
    count = 0;
    while(file.good() and count<1000){
        std::getline(file,line);
        count++;
        if(line.length()==0) break;
        
        std::vector<std::string> row;
        boost::tokenizer<boost::escaped_list_separator<char> > tokenizer(line,separator);
        for(auto i=tokenizer.begin();i!=tokenizer.end();++i){
            std::string item = *i;
            boost::trim(item);
            row.push_back(item);
        }
        
        for(unsigned int i=0;i<names.size();i++){
            if(row.size()<=i) break;
            std::string value = row[i];
            //Don't attempt to lexical cast empty strings
            if(value.length()==0) continue;
            try{
                boost::lexical_cast<int>(value);
            }
            catch(boost::bad_lexical_cast){
                flags[i] = 1;
                try{
                    boost::lexical_cast<double>(value);
                }
                catch(boost::bad_lexical_cast){
                    flags[i] = 2;
                }
            }
        }
    }
    // Clear flags on file (e.g. in case we got to end of file) and go back to start of data
    file.clear();
    file.seekg(position);
    
    //Determine types based on lexical cast flags
    std::vector<std::string> types(names.size());
    for(unsigned int i=0;i<names.size();i++){
        switch(flags[i]){
            case 0: types[i] = "INTEGER"; break;
            case 1: types[i] = "REAL"; break;
            case 2: types[i] = "TEXT"; break;
            default: types[i] = "TEXT"; break;
        }
    }
    
    // Create temporary table
    std::string temp_name = "stencila_"+name+"_temp";
    execute("DROP TABLE IF EXISTS \""+temp_name+"\"");
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
    
    execute("BEGIN TRANSACTION");
    count = 0;
    while(file.good()){
        
        std::getline(file,line);
        count++;
        if(line.length()==0) break;
        
        std::vector<std::string> row;
        boost::tokenizer<boost::escaped_list_separator<char> > tokenizer(line,separator);
        for(auto i=tokenizer.begin();i!=tokenizer.end();++i){
            std::string item = *i;
            boost::trim(item);
            row.push_back(item);
        }
        
        //Check that row is the correct size
        if(row.size()!=names.size()) 
            STENCILA_THROW(Exception,boost::str(boost::format("Line %i has %i items but expected %i items")%(count+1)%row.size()%names.size()));
        
        for(unsigned int i=0;i<names.size();i++){
            insert_cursor.bind(i+1,row[i]);
        }
        
        insert_cursor.execute();
        insert_cursor.reset();
    }
    file.close();
    execute("END TRANSACTION");
    
    //! Replace the existing table with the new one
    execute("DROP TABLE IF EXISTS \""+name+"\"");
    execute("ALTER TABLE \"stencila_"+name+"_temp\" RENAME TO \""+name+"\"");
    
    return table(name);
}

//! @brief 
//! @param name
//! @return 
Datatable Dataset::import(const std::string& name){
    //Check to see if this Datatable is already registered
    if(not value<int>("SELECT count(*) FROM stencila_datatables WHERE name==?",name)){
        execute("INSERT INTO stencila_datatables(name,source,status) VALUES(?,'table',2)",name);
    }
    
    //! @brief 
    //! @param name
    //! @return 
    return table(name);
}

//! @brief 
//! @param sql
//! @param reuse
//! @return 
Datatable Dataset::select(const std::string& sql, bool reuse){
    std::string signature = boost::lexical_cast<std::string>(Hash(sql));
    
    bool exists = value<int>("SELECT count(*) FROM stencila_datatables WHERE signature=="+signature);

    // If not reusing cached table then drop existing table if it exists
    if(exists and !reuse) {
        std::string name = value<std::string>("SELECT name FROM stencila_datatables WHERE signature=="+signature);
        drop(name);
    }
    
    // If does not yet exist then create the table
    if(!exists){
        //The extra "SELECT * FROM" in the following SQL is necessary for the correct interpretation of quoted
        //field names in the original SQL. If you don't treat the orginal SQL as a subquery then statements like
        // "SELECT "year",..." get treated as a string "year".
        std::string name = "stencila_"+boost::lexical_cast<std::string>(Hash());
        execute("CREATE TEMPORARY TABLE \""+name+"\" AS SELECT * FROM ("+sql+ ")");
        execute("INSERT INTO stencila_datatables(name,source,sql,signature,status) VALUES(?,'select',?,?,0)",name,sql,signature);
        return table(name);
    } else {
        std::string name = value<std::string>("SELECT name FROM stencila_datatables WHERE signature=="+signature);
        return table(name);
    }
    
    STENCILA_THROW(Exception,"This line should never be reached");
}

//! @brief 
//! @param original
//! @return 
Datatable Dataset::clone(const std::string& original){
    std::string signature = boost::lexical_cast<std::string>(Hash());
    std::string name = "stencila_"+signature;
    
    execute("DROP TABLE IF EXISTS \""+name+"\"");
    execute("CREATE TEMPORARY TABLE \""+name+"\" AS SELECT * FROM \""+original+"\"");
    execute("INSERT INTO stencila_datatables(name,source,sql,signature,status) VALUES(?,'clone',?,?,0)",name,original,signature);
    
    return table(name);
}

}
