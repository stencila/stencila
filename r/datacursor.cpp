#include <stencila/datacursor.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_FUNC Datacursor_fetch(SEXP self){
	STENCILA_R_BEGIN
		return wrap(
			from<Datacursor>(self).fetch()
		);
	STENCILA_R_END
}