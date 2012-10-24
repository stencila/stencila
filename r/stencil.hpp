#include <stencila/stencil.hpp>
using namespace Stencila;

inline std::string Stencil_callback(std::string handler){
    /*
    * Call an R handler function using Rcpp::Language which provides a much
    * easier interface than using the 'eval' R API function e.g. eval(name,R_GlobalEnv)
    */
	STENCILA_R_TRY
        Rcpp::Language call(handler,"attr","value");
        return as<std::string>(call.eval());
	STENCILA_R_CATCH
}

class RContext : public Context<RContext> {
private:

    std::string call(const std::string& handler,const std::string& arg1){
        /*
        * Call an R handler function using Rcpp::Language which provides a much
        * easier interface than using the 'eval' R API function e.g. eval(name,R_GlobalEnv)
        */
        STENCILA_R_TRY
            Rcpp::Language call(handler,arg1);
            return as<std::string>(call.eval());
        STENCILA_R_CATCH
    }


public:

    RContext(void){
    }

    std::string text(const std::string& expression){
        return call("canvas_text", expression);
    }
};

STENCILA_R_FUNC Stencil_new(void){
	STENCILA_R_BEGIN
		return STENCILA_R_TO(Stencil,new Stencil);
	STENCILA_R_END
}

STENCILA_R_FUNC Stencil_load(SEXP self, SEXP content){
	STENCILA_R_BEGIN
		from<Stencil>(self).load(as<std::string>(content));
		return nil;
	STENCILA_R_END
}

STENCILA_R_FUNC Stencil_dump(SEXP self){
	STENCILA_R_BEGIN
		return wrap(from<Stencil>(self).dump());
	STENCILA_R_END
}

STENCILA_R_FUNC Stencil_render(SEXP self){
	STENCILA_R_BEGIN
		RContext context;
        from<Stencil>(self).render(context);
        return nil;
	STENCILA_R_END
}
