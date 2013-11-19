//! @file format.hpp
//! @brief Definition of function format
//! @author Nokome Bentley

#pragma once

#include <string>

#include <boost/format.hpp>

namespace Stencila {
    
class Format : public boost::format {
public:

    Format(const std::string pattern):
        boost::format(pattern){
    }
    
    operator std::string (void){
        return str();
    }
    
    template<typename Type>
    Format& operator<<(const Type& type){
        boost::format::operator%(type);
        return *this;
    }
};



}
