#pragma once

#include <string>
#include <sstream>
#include <ostream>

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

	std::string message(void) const {
		return message_;
	}

	const char* what(void) const throw() {
		std::ostringstream stream;
		if(file_){
			stream << message_ << "\n  location: " << file_ << " " << line_;
		} else {
			stream << message_;
		}
		return stream.str().c_str();
	}
};

inline std::ostream& operator<<(std::ostream& stream,const Exception& exception){
	stream<<exception.what();
	return stream;
}

#define STENCILA_THROW(exception,message) throw exception(message,__FILE__,__LINE__);

}


