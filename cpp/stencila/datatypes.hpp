//! @file datatypes.hpp
//! @brief Definition of data types
//! @author Nokome Bentley

#pragma once

#include <string>

namespace Stencila {

//! @class Datatype
//! @todo Document fully
class Datatype {

public:
    char code;

    Datatype(char value=0):
    code(value){
    }
    
    //! @brief 
    //! @param other
    //! @return 
    bool operator==(const Datatype& other) const {
        return code==other.code;
    }

    //! @brief 
    //! @param other
    //! @return 
    bool operator!=(const Datatype& other) const {
        return code!=other.code;
    }

    //! @brief 
    //! @return 
    std::string name(void) const {
        switch(code){
            case 'n': return "Null";
            case 'i': return "Integer";
            case 'r': return "Real";
            case 't': return "Text";
        }
        return "Undefined";
    }

    //! @brief 
    //! @return 
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

const Datatype Null('n');
const Datatype Integer('i');
const Datatype Real('r');
const Datatype Text('t');

}
