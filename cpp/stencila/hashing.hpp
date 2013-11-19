//! @file hashing.hpp
//! @brief Declaration of a hash function
//! @author Nokome Bentley

#pragma once

#include <smhasher/MurmurHash3.h>

namespace Stencila {

//! Create a hash of string using MurmurHash3
//! (See http://en.wikipedia.org/wiki/MurmurHash, http://code.google.com/p/smhasher/wiki/MurmurHash3)
unsigned int Hash(const std::string& string = "");

}
