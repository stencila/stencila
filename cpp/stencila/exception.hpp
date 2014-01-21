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

    Exception(std::string message="",const char* file=0, int line=0):
        message_(message),
        file_(file),
        line_(line){
    }

    ~Exception(void) throw() {
    }

    const char* what(void)  const throw() {
        std::ostringstream stream;
        std::string filename = "";
        if(file_) filename = boost::filesystem::path(file_).filename().string();
        stream << filename << ":" << line_ << ":" << message_;
        return stream.str().c_str();
    }
};

inline std::ostream& operator<<(std::ostream& stream,const Exception& exception){
    stream<<exception.what();
    return stream;
}

class Unimplemented : public Exception {

public:

    Unimplemented(std::string what="",const char* file=0, int line=0):
        Exception("Unimplemented: "+what,file,line){
    }

    ~Unimplemented(void) throw() {
    }

};


#define STENCILA_THROW(exception,message) throw exception(message,__FILE__,__LINE__);

}


