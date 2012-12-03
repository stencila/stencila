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

//! @file dataset-math-functions.hpp
//! @brief Definition of math functions
//! Development influenced by Liam Healy's sqlite extension module at http://www.sqlite.org/contrib/download/extension-functions.c?get=25

#pragma once

#include <cmath>
#include <cassert>
#include <cerrno>
#include <cstring>

#include <stencila/sqlite.hpp>

namespace Stencila {
namespace MathFunctions {

#define STENCILA_LOCAL_0(name, function) \
static void name(sqlite3_context *context, int argc, sqlite3_value **argv){\
  assert( argc==0 );\
  sqlite3_result_double(context, function());\
}

#define STENCILA_LOCAL_1(name, function) \
static void name(sqlite3_context *context, int argc, sqlite3_value **argv){\
  assert( argc==1 );\
  if(sqlite3_value_type(argv[0])==SQLITE_NULL){\
      sqlite3_result_null(context);\
  } else {\
      double arg = sqlite3_value_double(argv[0]);\
      errno = 0;\
      double result = function(arg);\
      if(errno == 0) {\
        sqlite3_result_double(context, result);\
      } else {\
        sqlite3_result_error(context, std::strerror(errno), errno);\
      }\
  }\
}

#define STENCILA_LOCAL_2(name, function) \
static void name(sqlite3_context *context, int argc, sqlite3_value **argv){\
  assert( argc==2 );\
  if(sqlite3_value_type(argv[0])==SQLITE_NULL or sqlite3_value_type(argv[1])==SQLITE_NULL){\
      sqlite3_result_null(context);\
  } else {\
      double arg1 = sqlite3_value_double(argv[0]);\
      double arg2 = sqlite3_value_double(argv[1]);\
      errno = 0;\
      double result = function(arg1,arg2);\
      if(errno == 0) {\
        sqlite3_result_double(context, result);\
      } else {\
        sqlite3_result_error(context, std::strerror(errno), errno);\
      }\
  }\
}

//! @{
//! Trigonometric functions

STENCILA_LOCAL_1(cos, std::cos)
STENCILA_LOCAL_1(sin, std::sin)
STENCILA_LOCAL_1(tan, std::tan)
STENCILA_LOCAL_1(acos, std::acos)
STENCILA_LOCAL_1(asin, std::asin)
STENCILA_LOCAL_1(atan, std::atan)
//STENCILA_LOCAL_2(atan2, std::atan2)

//! @}

//! @{
//! Hyperbolic functions

STENCILA_LOCAL_1(cosh, std::cosh)
STENCILA_LOCAL_1(sinh, std::sinh)
STENCILA_LOCAL_1(tanh, std::tanh)

//! @}

//! @{
//! Exponential and logarithmic functions

STENCILA_LOCAL_1(exp, std::exp)
STENCILA_LOCAL_1(ln, std::log)
STENCILA_LOCAL_1(log, std::log)
STENCILA_LOCAL_1(log10, std::log10)

//! @}

//! Power functions

STENCILA_LOCAL_2(pow, std::pow)

static inline double squareFunc(double x){ return x*x;}
STENCILA_LOCAL_1(square, squareFunc)

STENCILA_LOCAL_1(sqrt, std::sqrt)

//! @}

//! Rounding, absolute value and remainder functions

// Implementation of sign based on answers at http://stackoverflow.com/q/1903954/1583041
static void sign(sqlite3_context *context, int argc, sqlite3_value **argv){
  switch( sqlite3_value_type(argv[0]) ){
    case SQLITE_NULL: {
      sqlite3_result_null(context);
      break;
    }
    case SQLITE_INTEGER: {
      int arg = sqlite3_value_int64(argv[0]);
      sqlite3_result_int64(context, (int(0) < arg) - (arg < int(0)));
      break;
    }
    default: {
      double arg = sqlite3_value_double(argv[0]);
      sqlite3_result_int64(context, (double(0) < arg) - (arg < double(0)));
      break;
    }
  }
}

STENCILA_LOCAL_1(fabs, std::fabs)

STENCILA_LOCAL_1(ceil, std::ceil)
STENCILA_LOCAL_1(floor, std::floor)


//! @}

const double Pi = 3.14159265358979323846;

static inline double piFunc(void){ return Pi;}
STENCILA_LOCAL_0(pi,piFunc)

static inline double radiansFunc(double x){ return x*Pi/180.0;}
STENCILA_LOCAL_1(radians, radiansFunc)

static inline double degreesFunc(double x){ return 180.0*x/Pi; }
STENCILA_LOCAL_1(degrees, degreesFunc)

#undef STENCILA_LOCAL_0
//#undef STENCILA_LOCAL_1
//#undef STENCILA_LOCAL_2

#define STENCILA_LOCAL(NAME,ARGS) \
    sqlite3_create_function(db, #NAME, ARGS, SQLITE_UTF8, 0, NAME, 0, 0);

inline void create(sqlite3* db) {
    //This list includes commented lines for builtin SQLite functions at http://www.sqlite.org/lang_corefunc.html
    //That is so this list can be used to constuct Dataquery call elements in R, Python etc packages

    STENCILA_LOCAL(cos,1)
    STENCILA_LOCAL(sin,1)
    STENCILA_LOCAL(tan,1)
    STENCILA_LOCAL(acos,1)
    STENCILA_LOCAL(asin,1)
    STENCILA_LOCAL(atan,1)
    STENCILA_LOCAL(atan,2)

    STENCILA_LOCAL(cosh,1)
    STENCILA_LOCAL(sinh,1)
    STENCILA_LOCAL(tanh,1)

    STENCILA_LOCAL(pi,0)
    STENCILA_LOCAL(degrees,1)
    STENCILA_LOCAL(radians,1)

    STENCILA_LOCAL(exp,1)
    STENCILA_LOCAL(ln,1)
    STENCILA_LOCAL(log,1)
    STENCILA_LOCAL(log10,1)

    STENCILA_LOCAL(pow,2)
    STENCILA_LOCAL(square,1)
    STENCILA_LOCAL(sqrt,1)

    //abs
    //round
    STENCILA_LOCAL(sign,1)
    STENCILA_LOCAL(ceil,1)
    STENCILA_LOCAL(floor,1)
    
    //random
}

#undef STENCILA_LOCAL

}
}
