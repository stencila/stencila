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

//! @file dataquery.hpp
//! @brief Definition of class Dataquery

#pragma once

#include <string>
#include <vector>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

#include "exception.hpp"

namespace Stencila {

//! @class Dataquery
//! @todo Document fully
class Dataquery {

private:
	
	std::vector<std::string> columns_;
	bool distinct_;
	std::vector<std::string> wheres_;
	std::vector<std::string> bys_;
	std::vector<std::string> havings_;
	std::vector<std::pair<std::string,int>> orders_;
	unsigned int limit_;
	unsigned int offset_;

public:
	
	Dataquery(void):
		distinct_(false),
		limit_(0),
		offset_(0){
	};
	
	Dataquery& columns(void){
		return *this;
	}
	
	Dataquery& columns(const std::string& expr){
		columns_.push_back(expr);
		return *this;
	}
	
	template<typename... Expressions>
	Dataquery& columns(const std::string& expr,const Expressions&... exprs){
		columns(expr);
		columns(exprs...);
		return *this;
	}
	
	Dataquery& distinct(bool value=true){
		distinct_ = value;
		return *this;
	};
	Dataquery& all(bool value=true){
		distinct_ = !value;
		return *this;
	};
	
	Dataquery& where(void){
		return *this;
	};
	template<typename... Expressions>
	Dataquery& where(const std::string& expr, const Expressions&... exprs){
		wheres_.push_back(expr);
		return where(exprs...);
	};
	
	Dataquery& by(void){
		return *this;
	};
	template<typename... Expressions>
	Dataquery& by(const std::string& expr, const Expressions&... exprs){
		bys_.push_back(expr);
		return by(exprs...);
	};
	
	Dataquery& having(void){
		return *this;
	};
	template<typename... Expressions>
	Dataquery& having(const std::string& expr, const Expressions&... exprs){
		havings_.push_back(expr);
		return having(exprs...);
	};
	
	Dataquery& order(const std::string& column, int direction=1){
		orders_.push_back({column,direction});
		return *this;
	}
	
	Dataquery& limit(unsigned int value){
		limit_ = value;
		return *this;
	}
	
	Dataquery& offset(unsigned int value){
		offset_ = value;
		return *this;
	}

	std::string sql(std::string from) const {
		
		std::vector<std::string> columns;
		for(auto i=bys_.begin();i!=bys_.end();i++){
			if(std::find(columns_.begin(), columns_.end(),*i)==columns_.end()){
				columns.push_back(*i);
			}
		}
		columns.insert(columns.end(),columns_.begin(),columns_.end());
		
		std::string sql = "SELECT";
		
		if(distinct_) sql += " DISTINCT";
		
		if(columns.size()==0) sql += " *";
		else {
			sql += " ";
			for(auto i=columns.begin();i!=columns.end();i++){
				sql +=  *i ;
				if(i!=columns.end()-1) sql += ", ";
			}
		}
		
		sql += " FROM "+from;
		
		if(wheres_.size()>0){
			sql += " WHERE ";
			if(wheres_.size()>1) sql += "(";
			for(auto i=wheres_.begin();i!=wheres_.end();i++){
				sql += *i;
				if(i!=wheres_.end()-1) sql += ") AND (";
			}
			if(wheres_.size()>1) sql += ")";
		}
		
		if(bys_.size()>0){
			sql += " GROUP BY ";
			for(auto i=bys_.begin();i!=bys_.end();i++){
				sql += *i;
				if(i!=bys_.end()-1) sql += ", ";
			}
		}
		
		if(havings_.size()>0){
			if(bys_.size()==0) throw Exception("A \"having\" clause is not permitted when there is no \"by\" clauses");
			sql += " HAVING ";
			if(havings_.size()>1) sql += "(";
			for(auto i=havings_.begin();i!=havings_.end();i++){
				sql += *i;
				if(i!=havings_.end()-1) sql += ") AND (";
			}
			if(havings_.size()>1) sql += ")";
		}
		
		if(orders_.size()>0){
			sql += " ORDER BY ";
			for(auto i=orders_.begin();i!=orders_.end();i++){
				auto order = *i;
				sql += order.first + ((order.second>0)?(" ASC"):(" DESC"));
				if(i!=orders_.end()-1) sql += ",";
			}
		}
		
		if(limit_>0){
			sql += " LIMIT " + boost::lexical_cast<std::string>(limit_);
		}
		
		if(offset_>0){
			//Offset can only come after a limit clause. So add one if not present.
			//The theoretical maximum number of rows in an SQLite database
			//is 2^64 = 18446744073709551616 (see http://www.sqlite.org/limits.html)
			//However SQLite bauks at such a large integer in an limit clause so instead
			//we have to use the maximum value for an integer: 2^64/2
			if(limit_==0) sql += " LIMIT 9223372036854775807";
			sql += " OFFSET " + boost::lexical_cast<std::string>(offset_);
		}
			
		return sql;
	}
	
};

template<typename... Expressions>
Dataquery get(const Expressions&... exprs){
	Dataquery dataquery;
	dataquery.columns(exprs...);
	return dataquery;
}

}