#include "../../../cpp/datacursor.hpp"
using namespace Stencila;

EXPORT SEXP Datacursor_fetch(SEXP self){
	BEGIN
		return wrap(
			from<Datacursor>(self).fetch()
		);
	END
}