#pragma once

#include <type_traits>
#include <tuple>

namespace Stencila {

/**
 * Function traits
 *
 * For a function type provide the following:
 *
 *  - `function_type` The complete type of the function
 *  - `return_type` The type returned
 *  - `arity` The number or arguments
 *  - `args<0...arity>::type` The type of each argument
 *
 * For a member function also provides:
 *
 *   - `owner_type` The type of the owning class
 *
 * Based on [KennyTM's](http://stackoverflow.com/a/7943765)[implementation](https://github.com/kennytm/utils/blob/master/traits.hpp)
 * See that code for refinements not applied here.
 */
template <typename Type>
struct FunctionTraits : public FunctionTraits<decltype(&Type::operator())> {};

template < 
	typename Return, 
	typename... Args
>
struct FunctionTraits<Return(Args...)> {
	enum { arity = sizeof...(Args) };

	typedef Return function_type(Args...);
	typedef Return return_type;

	template <size_t i>
	struct args {
		typedef typename std::tuple_element<i, std::tuple<Args...>>::type type;
	};
};

template <typename Return, typename... Args>
struct FunctionTraits<Return(*)(Args...)> : public FunctionTraits<Return(Args...)> {};

template <typename Class, typename Return, typename... Args>
struct FunctionTraits<Return(Class::*)(Args...)> : public FunctionTraits<Return(Args...)> {
	typedef Class& owner_type;
};

template <typename Class, typename Return, typename... Args>
struct FunctionTraits<Return(Class::*)(Args...) const> : public FunctionTraits<Return(Args...)> {
	typedef const Class& owner_type;
};

template <typename Class, typename Return, typename... Args>
struct FunctionTraits<Return(Class::*)(Args...) volatile> : public FunctionTraits<Return(Args...)> {
	typedef volatile Class& owner_type;
};

template <typename Class, typename Return, typename... Args>
struct FunctionTraits<Return(Class::*)(Args...) const volatile> : public FunctionTraits<Return(Args...)> {
	typedef const volatile Class& owner_type;
};

/**
 * @name "Has" traits
 *
 * Used to determine if a type *has* a particular method or member.
 * Belong to the [member detector idiom](http://en.wikibooks.org/wiki/More_C++_Idioms/Member_Detector).
 * 
 * @{
 */

struct HasTrait {
	typedef char (&yes)[1];
	typedef char (&no)[2];
};

template <typename Type>
struct HasCall : HasTrait {
	template <typename A> static yes test(decltype(&A::operator()));
	template <typename A> static no test(...);
	enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct HasBeginEnd : HasTrait {
	template<typename A, A, A> struct Match;
	// This must use const_iterator so that it is true for std::set<>
	template <typename A> static yes test(Match<typename A::const_iterator (A::*)() const,&A::begin,&A::end>*);
	template <typename A> static no test(...);
	enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct HasKeyTypeValueType : HasTrait {
	template <typename A> static yes test(typename A::key_type*,typename A::value_type*);
	template <typename A> static no test(...);
	enum {value = (sizeof(test<Type>(0,0)) == sizeof(yes))};
};

template <typename Type>
struct HasMappedType : HasTrait {
	template <typename A> static yes test(typename A::mapped_type*);
	template <typename A> static no test(...);
	enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct HasStructureType : HasTrait {
	template <typename A> static yes test(typename A::structure_type*);
	template <typename A> static no test(...);
	enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct HasArrayType : HasTrait {
	template <typename A> static yes test(typename A::array_type*);
	template <typename A> static no test(...);
	enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

/**
 * @}
 */


/**
 * @name "Is" traits
 *
 * Used to determine if a type *is* of a particular kind.
 * Usually determined as a combination of `Has` traits.
 * 
 * @{
 */

template <typename Type>
struct IsCallable : std::integral_constant<bool,
	HasCall<Type>::value
>{};

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

template <typename Type>
struct IsStructure : std::integral_constant<bool,
	std::is_class<Type>::value and 
	HasStructureType<Type>::value
>{};

template <typename Type>
struct IsArray : std::integral_constant<bool,
	std::is_class<Type>::value and 
	HasArrayType<Type>::value
>{};

}
