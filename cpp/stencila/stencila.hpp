/*
Copyright (c) 2012, Nokome Bentley, nokome.bentley@stenci.la

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//!	@file stencila.hpp
//! 	@brief Includes all Stencila header files. Also container for Doxygen mainpage command.

#pragma once

#include <vector>

#include "exception.hpp"
#include "printing.hpp"
#include "testing.hpp"

#include "datatypes.hpp"
#include "dataset.hpp"
#include "datacursor.hpp"
#include "datatable.hpp"

#include "dataset.cpp"

//!	@namespace Stencila
//!	@brief The namespace for all Stencila classes and functions
namespace Stencila {};

/*! 

@mainpage Main Page

@section introduction Introduction

The Stencila C++ library is the bas
  
@section requirements Requirements
 
@subsection boost Boost

Stencila makes extensive use of the <a href="http://www.boost.org/">Boost C++ libraries</a>.

@subsection sqlite SQLite

Stencila uses <a href="http://www.sqlite.org/">SQLite</a>.

*/