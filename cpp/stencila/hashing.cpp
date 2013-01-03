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
