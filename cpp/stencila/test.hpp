//! @file test.hpp
//! @brief Convienience macros and others used for testing
//! @author Nokome Bentley

#pragma once

template<class Type> 
void check_equal(std::vector<Type> a,std::vector<Type> b){
    BOOST_CHECK_EQUAL_COLLECTIONS(a.begin(),a.end(),b.begin(),b.end());
}
