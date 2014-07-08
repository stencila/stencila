#pragma once

#include <boost/algorithm/string.hpp>
#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

#include <stencila/exception.hpp>
#include <stencila/polymorph.hpp>

namespace Stencila {

template<class Derived>
class Mirror : public Polymorph<Derived> {
public:
    using Polymorph<Derived>::derived;

	template<typename Data,typename... Args>
	Derived& data(Data& data, Args... args){
		return derived();
	}

	template<typename Method,typename... Args>
	Derived& method(Method& method, Args... args){
		return derived();
	}
	
}; // class Mirror

template<class Type>
class Has : public Mirror<Has<Type>> {
private:
    std::string name_;
    bool has_;

public:

    Has(const std::string& name):
        name_(name),
        has_(false){
        static_cast<Type*>(0)->reflect(*this);
    }

 	template<typename Data,typename... Args>
	Has& data(Data& data, const std::string& name, Args... args){
		if(not has_) has_ = name==name_;
		return *this;
	}

	operator bool(void) const {
		return has_;
	}
  
};

template<class Type>
class RowHeader : public Mirror<RowHeader<Type>>, public std::string {
private:
    std::string separator_;

public:
    RowHeader(const std::string& separator="\t"):
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


class RowString : public Mirror<RowString>, public std::string {
private:
    std::string separator_;

public:
	template<class Type>
    RowString(Type& type, const std::string& separator="\t"):
        separator_(separator){
        type.reflect(*this);
    }

    template<typename Data>
    RowString& data(Data& data, const std::string& name=""){
        if(length()>0) append(separator_);
        append(boost::lexical_cast<std::string>(data));
        return *this;
    }
}; // class RowString


class RowParser : public Mirror<RowParser> {
private:
    std::vector<std::string> items_;
    uint index_ = 0;

public:
    RowParser(const std::string& row,const std::string& separator="\t"){
    	boost::split(items_,row,boost::is_any_of(separator));
    }

    template<typename Data>
    RowParser& data(Data& data, const std::string& name=""){
    	if(index_>=items_.size()) STENCILA_THROW(Exception,str(boost::format("Not enough elements in row; got <%s>, need at least <%s>")%items_.size()%index_));
        data = boost::lexical_cast<Data>(items_[index_]);
    	index_++;
        return *this;
    }
}; // class RowWriter

/**
 * Matches columns with 
 */
class AttributeMatcher : public Mirror<AttributeMatcher> {
private:
	std::vector<std::string> names_;
	std::vector<std::string> values_;

public:
    AttributeMatcher(void){
    }

    AttributeMatcher(const std::string& names,const std::string& values, const std::string& separator="\t"){
    	this->names(names);
    	this->values(values);
    	if(names_.size()!=values_.size()) STENCILA_THROW(Exception,str(boost::format("Different numbers of names and values; got <%s> names and <%s> values using separator <%s>")%names_.size()%values_.size()%separator));
    }

    AttributeMatcher& names(const std::string& names, const std::string& separator="\t"){
    	boost::split(names_,names,boost::is_any_of(separator));
        return *this;
    }

    AttributeMatcher& values(const std::string& values, const std::string& separator="\t"){
    	boost::split(values_,values,boost::is_any_of(separator));
        return *this;
    }    

    template<typename Data>
    AttributeMatcher& data(Data& data, const std::string& name=""){
    	auto iter =  std::find(names_.begin(), names_.end(), name);
    	if(iter!=names_.end()){
    		uint index = iter-names_.begin();
    		data = boost::lexical_cast<Data>(values_[index]);
    	}
        return *this;
    }
}; // class AttributeMatcher

} // namespace Stencila
