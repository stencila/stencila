/*
Copyright (c) 2013 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

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
