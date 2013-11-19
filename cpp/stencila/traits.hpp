//! @file datacursor.hpp
//! @brief Definition of class Datacursor
//! @author Nokome Bentley

#pragma once

#include <type_traits>

namespace Stencila {
namespace Traits {

struct Has {
    typedef char (&yes)[1];
    typedef char (&no)[2];
};

template <typename Type>
struct HasBeginEnd : Has {
    template<typename A, A, A> struct Match;
    // This must use const_iterator so that it is true for std::set<>
    template <typename A> static yes test(Match<typename A::const_iterator (A::*)() const,&A::begin,&A::end>*);
    template <typename A> static no test(...);
    enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct HasKeyTypeValueType : Has {
    template <typename A> static yes test(typename A::key_type*,typename A::value_type*);
    template <typename A> static no test(...);
    enum {value = (sizeof(test<Type>(0,0)) == sizeof(yes))};
};

template <typename Type>
struct HasMappedType : Has {
    template <typename A> static yes test(typename A::mapped_type*);
    template <typename A> static no test(...);
    enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct IsContainer : std::integral_constant<bool,
    std::is_class<Type>::value and 
    HasBeginEnd<Type>::value
>{};

template <typename Type>
struct IsAssociative : std::integral_constant<bool,
    IsContainer<Type>::value and 
    HasKeyTypeValueType<Type>::value
>{};

template <typename Type>
struct IsPaired : std::integral_constant<bool,
    IsAssociative<Type>::value and 
    HasMappedType<Type>::value
>{};

}
}
