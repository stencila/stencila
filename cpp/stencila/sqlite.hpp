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

//!	@file sqlite.hpp
//!	@brief Some convienience functions for using SQLite

#pragma once

#include <sqlite3.h>

#include "exception.hpp"

namespace Stencila {
	
class SqliteException : public Exception {
protected:

	int code_;
	
public:	

	SqliteException(int code,std::string message,const char* file=0, int line=0):
		Exception(message,file,line),
		code_(code){		
	}
	
	//! @see These descriptions are from http://www.sqlite.org/c3ref/c_abort.html
	const char* description(void) const {
		switch(code_){
			case SQLITE_OK: return "No error";
			case SQLITE_ERROR: return "SQL error or missing database";
			case SQLITE_INTERNAL: return "Internal logic error in SQLite";
			case SQLITE_PERM: return "Access permission denied";
			case SQLITE_ABORT: return "Callback routine requested an abort";
			case SQLITE_BUSY: return "The database file is locked";
			case SQLITE_LOCKED: return "A table in the database is locked";
			case SQLITE_NOMEM: return "A malloc() failed";
			case SQLITE_READONLY: return "Attempt to write a readonly database";
			case SQLITE_INTERRUPT: return "Operation terminated by sqlite3_interrupt()";
			case SQLITE_IOERR: return "Some kind of disk I/O error occurred";
			case SQLITE_CORRUPT:  return "The database disk image is malformed";
			case SQLITE_NOTFOUND: return "Unknown opcode in sqlite3_file_control()";
			case SQLITE_FULL: return "Insertion failed because database is full";
			case SQLITE_CANTOPEN : return "Unable to open the database file";
			case SQLITE_PROTOCOL: return "Database lock protocol error";
			case SQLITE_EMPTY: return "Database is empty";
			case SQLITE_SCHEMA: return "The database schema changed";
			case SQLITE_TOOBIG: return "String or BLOB exceeds size limit";
			case SQLITE_CONSTRAINT: return "Abort due to constraint violation";
			case SQLITE_MISMATCH: return "Data type mismatch";
			case SQLITE_MISUSE: return "Library used incorrectly";
			case SQLITE_NOLFS: return "Uses OS features not supported on host";
			case SQLITE_AUTH: return "Authorization denied";
			case SQLITE_FORMAT: return "Auxiliary database format error";
			case SQLITE_RANGE: return "2nd parameter to sqlite3_bind out of range";
			case SQLITE_NOTADB: return "File opened that is not a database file";
			case SQLITE_ROW: return "sqlite3_step() has another row ready";
			case SQLITE_DONE: return "sqlite3_step() has finished executing";
		}
		return "Unknown error";
	}
	
	const char* what(void)  const throw() {		
		std::ostringstream stream;
        stream << file_ << ":" << line_ << ": " << description() << "(" << code_ << "): " << message_;
		return stream.str().c_str();
	}
};
	
#define STENCILA_SQLITE_THROW(db,code)\
	throw SqliteException(code,sqlite3_errmsg(db),__FILE__,__LINE__);

#define STENCILA_SQLITE_TRY(db,call) {\
	int code = call;\
	if(code!=SQLITE_OK) { STENCILA_SQLITE_THROW(db,code); }\
}

}