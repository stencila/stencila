//! @file hashing.cpp
//! @brief Implementation of a hash function
//! @author Nokome Bentley
//! This file exists primarily because MurmurHash3.cpp cannot be included in hashing.hpp without "multiple definition" compiler errors

#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/lexical_cast.hpp>

#include <smhasher/MurmurHash3.cpp>

namespace Stencila {

//! Create a hash from a string key. If no key is supplied then create a hash from a randomly generated UUID
unsigned int Hash(const std::string& string){
    unsigned int hash;
    std::string key = string;
    if(key.length()==0){
        boost::uuids::uuid uuid = boost::uuids::random_generator()();
        key = boost::lexical_cast<std::string>(uuid);
    }
    MurmurHash3_x86_32(key.c_str(),key.length(),0,&hash);
    return hash;
}

}
