#pragma once

#include <fstream>

#include <boost/algorithm/string.hpp>
#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

#include <stencila/exception.hpp>
#include <stencila/mirror.hpp>

namespace Stencila {
namespace Mirrors {

class RowHeader : public Mirror<RowHeader>, public std::string {
private:
    std::string separator_;

public:
    template<class Type>
    RowHeader(const Type& type, const std::string& separator="\t"):
        separator_(separator){
        static_cast<Type*>(0)->reflect(*this);
    }

    template<typename Data>
    RowHeader& data(Data& data, const std::string& name=""){
        if(length()>0) append(separator_);
        append(name);
        return *this;
    }
}; // class RowHeader


class RowGenerator : public Mirror<RowGenerator>, public std::string {
private:
    std::string separator_;

public:
	template<class Type>
    RowGenerator(Type& type, const std::string& separator="\t"):
        separator_(separator){
        type.reflect(*this);
    }

    template<typename Data>
    RowGenerator& data(Data& data, const std::string& name=""){
        if(length()>0) append(separator_);
        append(str(boost::format("%s")%data));
        return *this;
    }
}; // class RowGenerator

class RowWriter : public Mirror<RowWriter>, public std::ofstream {
private:
    bool all_;
    bool started_;
    std::string path_;
    std::vector<std::string> prefixes_;
    std::vector<std::string> names_;
    std::string separator_;
    std::ofstream file_;

    template<
        typename Arg,
        typename... Args
    >
    void write_prefixes_(Arg arg, Args... args){
        file_<<arg<<separator_;
        started_ = true;
        write_prefixes_(args...);
    }

    void write_prefixes_(void){
    }

public:

    template<class Type>
    RowWriter(const Type& type, const std::string& path, const std::vector<std::string>& prefixes = {}, const std::vector<std::string>& names = {},const std::string& separator="\t"):
        path_(path),
        prefixes_(prefixes),
        names_(names),
        separator_(separator),
        file_(path){
        // All `data` attributes?
        all_ = names.size()==0;
        // Write the header row
        for(auto prefix : prefixes_) file_<<prefix<<separator_;
        if(all_) file_<<RowHeader(type,separator);
        else {
            for(auto name : names) file_<<name<<separator_;
        }
        file_<<"\n";
    }

    template<typename Data>
    RowWriter& data(Data& data, const std::string& name=""){
        bool write = false;
        if(all_) write = true;
        else write = std::find(names_.begin(), names_.end(), name) != names_.end();
        if(write){
            if(started_) file_<<separator_;
            else started_ = true;
            file_<<data;
        }
        return *this;
    }

    template<class Reflector, typename... Args>
    RowWriter& write(Reflector& reflector, Args... args){
        started_ = false;
        write_prefixes_(args...);
        reflector.reflect(*this);
        file_<<"\n";
        return *this;
    }

}; // class RowWriter

class RowParser : public Mirror<RowParser> {
private:
    std::vector<std::string> items_;
    unsigned int index_ = 0;

public:
    template<class Type>
    RowParser(Type& type, const std::string& row,const std::string& separator="\t"){
    	boost::split(items_,row,boost::is_any_of(separator));
        type.reflect(*this);
    }

    template<typename Data>
    RowParser& data(Data& data, const std::string& name=""){
    	if(index_>=items_.size()) STENCILA_THROW(Exception,str(boost::format("Not enough elements in row; got <%s>, need at least <%s>")%items_.size()%index_));
        data = boost::lexical_cast<Data>(items_[index_]);
    	index_++;
        return *this;
    }
}; // class RowParser

class ColumnMatcher : public Mirror<ColumnMatcher> {
private:
	std::vector<std::string> names_;
	std::vector<std::string> values_;

public:
    ColumnMatcher(void){
    }

    ColumnMatcher(const std::string& names,const std::string& values, const std::string& separator="\t"){
    	this->names(names);
    	this->values(values);
    	if(names_.size()!=values_.size()) STENCILA_THROW(Exception,str(boost::format("Different numbers of names and values; got <%s> names and <%s> values using separator <%s>")%names_.size()%values_.size()%separator));
    }

    ColumnMatcher& names(const std::string& names, const std::string& separator="\t"){
    	boost::split(names_,names,boost::is_any_of(separator));
        return *this;
    }

    ColumnMatcher& values(const std::string& values, const std::string& separator="\t"){
    	boost::split(values_,values,boost::is_any_of(separator));
        return *this;
    }    

    template<typename Data>
    ColumnMatcher& data(Data& data, const std::string& name=""){
    	auto iter =  std::find(names_.begin(), names_.end(), name);
    	if(iter!=names_.end()){
    		unsigned int index = iter-names_.begin();
    		data = boost::lexical_cast<Data>(values_[index]);
    	}
        return *this;
    }
}; // class ColumnMatcher

} // namepace Mirrors
} // namespace Stencila
