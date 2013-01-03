/*
Copyright (c) 2012, Nokome Bentley, nokome.bentley@stenci.la

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//!                        @file exception.hpp
//!                        @brief Definition of class Exception
//!                        @author Nokome Bentley

#pragma once

#include <string>
#include <sstream>
#include <ostream>

#include <boost/filesystem.hpp>

namespace Stencila {
                        
class Exception : public std::exception {
                        
protected:
                        
                        std::string message_;
                        const char* file_;
                        int line_;

public:
                        
                        //! @brief 
                        //! @param file
                        //! @param line
                        //! @return 
                        Exception(std::string message="",const char* file=0, int line=0):
                                                message_(message),
                                                file_(file),
                                                line_(line){                                                
                        }
    
    ~Exception(void) throw() {
    }
                        
                        //! @brief 
                        //! @return 
                        const char* what(void)  const throw() {
                                                std::ostringstream stream;
        stream << boost::filesystem::path(file_).filename().string() << ":" << line_ << ":" << message_;
                                                return stream.str().c_str();
                        }
};
//! @brief 
//! @param exception
//! @return 
inline std::ostream& operator<<(std::ostream& stream,const Exception& exception){
                        stream<<exception.what();
                        return stream;
}

}

#define STENCILA_THROW(exception,message) throw exception(message,__FILE__,__LINE__);
