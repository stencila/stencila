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

//! @file query-cxx.hpp
//! @brief Function definitions for C++ embedded query language
//! @author Nokome Bentley

#pragma once

#include <stencila/tables/query.hpp>

namespace Stencila {
namespace Tables {

//! @namespace DQL
//! @brief Data Query Langauge for C++
//!
//! Includes several functions for convieniently defining Dataqueries within C++
//! Instead of creating and linking Query elements individually, these functions
//! provide useful shortcuts. For example...
//! @todo Finish this documentation
namespace DQL {

//! @name Local convieience functions
//! @{

Constant<int>* convert(const int& value){
    return new Constant<int>(value);
}
Constant<float>* convert(const float& value){
    return new Constant<float>(value);
}
Constant<std::string>* convert(const std::string& value){
    return new Constant<std::string>(value);
}
template<class Element>
Element* convert(const Element& element){
    return new Element(element);
}

template<class Element>
void append(Element& element){
} 
template<class Element1,class Element2,class... Elements>
void append(Element1& element1,const Element2& element2,const Elements&... elements){
      element1.append(convert(element2));
      append(element1,elements...);
} 

//! @}

//! @name Column and As
//! @{

Column column(const std::string& name){
    return Column(name);
}

template<class Element>
As as(const std::string& name,const Element& element){
    return As(name,convert(element));
}

//! @}

//! @name Unary operators
//! @{

#define UNOP(name,symbol) \
      template<class Element> \
      name operator symbol(const Element& expr){ \
            return name(convert(expr)); \
      }

UNOP(Positive,+)
UNOP(Negative,-)
UNOP(Not,!)

#undef UNOP

//! @}

//! @name Binary operators
//! @{

#define BINOP(name,symbol) \
      template<class Left,class Right> \
      name operator symbol(const Left& left,const Right& right){ \
            return name(convert(left),convert(right)); \
      }

BINOP(Multiply,*)
BINOP(Divide,/)
BINOP(Add,+)
BINOP(Subtract,-)

BINOP(Equal,==)
BINOP(NotEqual,!=)
BINOP(LessThan,<)
BINOP(LessEqual,<=)
BINOP(GreaterThan,>)
BINOP(GreaterEqual,>=)

BINOP(And,&&)
BINOP(Or,||)

#undef BINOP

template<class Element>
In in(const Element& element,const std::vector<std::string> set){
    return In(convert(element),set);
}

//! @}

//! @name Function calls
//! @{

#define CALL(name) \
      template<class... Elements> \
      Call name(const Elements&... exprs){ \
            Call call(#name); \
            append(call,exprs...); \
            return call; \
      } 

#define CALL_0(name) \
      template<class Element> \
      Call name(void){ \
            return Call(#name); \
      } 

#define CALL_1(name) \
      template<class Element> \
      Call name(const Element& expr){ \
            return Call(#name,convert(expr)); \
      } 
      
#define CALL_2_M(name) \
      template<class Element1,class Element2,class... Elements> \
      Call name(const Element1& expr1,const Element2& expr2,const Elements&... exprs){ \
            Call call(#name,convert(expr1),convert(expr2)); \
            append(call,exprs...); \
            return call; \
      } 
    
#define AGG_1(name) \
      template<class Element> \
      Aggregate name(const Element& expr){ \
            return Aggregate(#name,convert(expr)); \
      } 

//! Number functions
//! See http://www.sqlite.org/lang_corefunc.html
CALL_1(abs)
CALL_2_M(max)
CALL_2_M(min)
CALL_0(random)
CALL_1(round)

// Number aggregate functions
//! See http://www.sqlite.org/lang_aggfunc.html
AGG_1(avg)
AGG_1(count)
AGG_1(max)
AGG_1(min)
AGG_1(sum)
AGG_1(mean)
AGG_1(geomean)
AGG_1(harmean)

// Text functions
//! See http://www.sqlite.org/lang_corefunc.html
CALL_1(length)
CALL_1(lower)
CALL_1(upper)
CALL_1(trim)
CALL_1(ltrim)
CALL_1(rtrim)
CALL(replace)
CALL(substr)

// Date and time functions
//! See http://www.sqlite.org/lang_datefunc.html
template<class Format, class Element, class Modifier>
Call strftime(const Format& format,const Element& expr, const Modifier& modifier){
      return Call("strftime",wrap(format),wrap(expr),wrap(modifier));
} 

#undef CALL_0
#undef CALL_1

//! @}

const Distinct distinct;
const All all;

template<class Element>
Where where(const Element& element){
      return Where(convert(element));
}

template<class Element>
By by(const Element& element){
      return By(convert(element));
}

template<class Element>
Having having(const Element& element){
      return Having(convert(element));
}

template<class Element>
Order order(const Element& element, float direction = 1){
      return Order(convert(element),direction);
}

Limit limit(unsigned int number){
      return Limit(number);
}

Offset offset(unsigned int number){
      return Offset(number);
}

template<class Element1,class Element2>
Top top(const Element1& element1,const Element2& element2,unsigned int num){
      return Top(convert(element1),convert(element2),num);
}

Margin margin(void){
      return Margin();
}

template<class Element>
Margin margin(const Element& element){
      return Margin(convert(element));
}

//! @name Adjusters
//! @{


template<class Value>
Proportion prop(const Value& value){
    return Proportion(convert(value));
}

template<class Value, class By>
Proportion prop(const Value& value, const By& by){
    Proportion prop(convert(value));
    prop.bys_append(convert(by));
    return prop;
}

//! @}

void query_append(Query& query){
}

template<class Element,class... Elements>
void query_append(Query& query,const Element& element, const Elements&... elements){
    query.append(convert(element));
    query_append(query,elements...);
}

template<class... Elements>
Query query(const Elements&... elements){
    Query query;
    query_append(query,elements...);
    return query;
}

}
}
}