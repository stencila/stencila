#pragma once

#include <string>

namespace Stencila {

class Datatype {
public:
    char code;

    Datatype(char value=0):
        code(value){
    }
    
    bool operator==(const Datatype& other) const {
        return code==other.code;
    }
 
    bool operator!=(const Datatype& other) const {
        return code!=other.code;
    }

    std::string name(void) const {
        switch(code){
            case 'n': return "Null";
            case 'i': return "Integer";
            case 'r': return "Real";
            case 't': return "Text";
        }
        return "Undefined";
    }

    operator std::string(void){
        return name();
    }

    std::string sql(void) const {
        switch(code){
            case 'n': return "NULL";
            case 'i': return "INTEGER";
            case 'r': return "REAL";
            case 't': return "TEXT";
        }
        return "NULL";
    }
};

static const Datatype Null('n');
static const Datatype Integer('i');
static const Datatype Real('r');
static const Datatype Text('t');

}
