#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_directives_quick)

BOOST_AUTO_TEST_CASE(strip){
	Stencil s(R"(html://
		
		`exec` directive elements shoul be removed completely

		<pre data-exec="" id="exec"></pre>


		Directive attributes should be removed

		<div data-if=""></div>
		<div data-switch="">
			<div data-case=""></div>
			<div data-case=""></div>
		</div>


		Flag attributes should be removed

		<div data-hash=""></div>
		<div data-out=""></div>
	)");

	BOOST_CHECK(s.select("pre#exec"));
	BOOST_CHECK(s.select("[data-if]"));
	BOOST_CHECK(s.select("[data-switch]"));
	BOOST_CHECK(s.select("[data-case]"));
	BOOST_CHECK(s.select("[data-hash]"));
	BOOST_CHECK(s.select("[data-out]"));

	s.strip();

	BOOST_CHECK(not s.select("pre#exec"));
	BOOST_CHECK(not s.select("[data-if]"));
	BOOST_CHECK(not s.select("[data-switch]"));
	BOOST_CHECK(not s.select("[data-case]"));
	BOOST_CHECK(not s.select("[data-hash]"));
	BOOST_CHECK(not s.select("[data-out]"));
}

BOOST_AUTO_TEST_CASE(exec){
	typedef Stencil::Execute E;
	{
		E e("py");
		BOOST_CHECK_EQUAL(e.contexts.size(),1);
		BOOST_CHECK_EQUAL(e.contexts[0],"py");
	}{
		E e("r,py");
		BOOST_CHECK_EQUAL(e.contexts.size(),2);
		BOOST_CHECK_EQUAL(e.contexts[0],"r");
		BOOST_CHECK_EQUAL(e.contexts[1],"py");
	}{
		E e("r,   py");
		BOOST_CHECK_EQUAL(e.contexts.size(),2);
		BOOST_CHECK_EQUAL(e.contexts[0],"r");
		BOOST_CHECK_EQUAL(e.contexts[1],"py");
	}{
		try {
			E e("r,bf");
		} catch(const Stencil::DirectiveException& exc){
			BOOST_CHECK_EQUAL(exc.type,"context-invalid");
		}  
	}

	{
		E e("r format text");
		BOOST_CHECK_EQUAL(e.format.expr,"text");
	}{
		E e("r format png");
		BOOST_CHECK_EQUAL(e.format.expr,"png");
	}{
		E e("r format svg");
		BOOST_CHECK_EQUAL(e.format.expr,"svg");
	}{
		try {
			E e("r format gnp");
		} catch(const Stencil::DirectiveException& exc){
			BOOST_CHECK_EQUAL(exc.type,"format-invalid");
		}  
	}

	{
		E e("r format png width 19");
		BOOST_CHECK_EQUAL(e.width.expr,"19");
	}

	{
		E e("py,r format png width 10 units cm size 4.2x8.4in");
		BOOST_CHECK_EQUAL(e.contexts.size(),2);
		BOOST_CHECK_EQUAL(e.contexts[0],"py");
		BOOST_CHECK_EQUAL(e.contexts[1],"r");
		BOOST_CHECK_EQUAL(e.format.expr,"png");
		BOOST_CHECK_EQUAL(e.size.expr,"4.2x8.4in");
	}{
		try {
			E e("r format png size 10x10km");
		} catch(const Stencil::DirectiveException& exc){
			BOOST_CHECK_EQUAL(exc.type,"units-invalid");
			BOOST_CHECK_EQUAL(exc.data,"km");
		}
	}

	{
		E e("r");
		BOOST_CHECK(not e.constant);
	}
	{
		E e("r const");
		BOOST_CHECK(e.constant);
	}

	{
		E e("cila");
		BOOST_CHECK(not e.show);
	}
	{
		E e("cila show");
		BOOST_CHECK(e.show);
	}

}

BOOST_AUTO_TEST_CASE(attr){
	Stencil::Attr attr;

	attr.parse("name value expression");
	BOOST_CHECK_EQUAL(attr.name,"name");
	BOOST_CHECK_EQUAL(attr.expression,"expression");
}

BOOST_AUTO_TEST_CASE(where){
	Stencil::Where where;

	where.parse("r");
	BOOST_CHECK_EQUAL(where.contexts.size(),1);
	BOOST_CHECK_EQUAL(where.contexts[0],"r");

	where.parse("r,py ,  cpp");
	BOOST_CHECK_EQUAL(where.contexts.size(),3);
	BOOST_CHECK_EQUAL(where.contexts[0],"r");
	BOOST_CHECK_EQUAL(where.contexts[1],"py");
	BOOST_CHECK_EQUAL(where.contexts[2],"cpp");
}

BOOST_AUTO_TEST_CASE(for_){
	typedef Stencil::For F;
	{
		F f("item in items");
		BOOST_CHECK_EQUAL(f.item,"item");
		BOOST_CHECK_EQUAL(f.items,"items");
	}{
		try{
			F f("foo bar");
		} catch(const Stencil::DirectiveException& exc){
			BOOST_CHECK_EQUAL(exc.type,"syntax");
		}
	}
}

BOOST_AUTO_TEST_CASE(par){
	typedef Stencil::Parameter P;
	{
		P p("x");
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"");
		BOOST_CHECK_EQUAL(p.value,"");
	}{
		P p("x type number");
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"number");
		BOOST_CHECK_EQUAL(p.value,"");
	}{
		P p("x type number value 42");
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"number");
		BOOST_CHECK_EQUAL(p.value,"42");
	}{
		P p("x value 42");
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"");
		BOOST_CHECK_EQUAL(p.value,"42");
	}{
		P p("x value pi*7*6");
		BOOST_CHECK_EQUAL(p.name,"x");
		BOOST_CHECK_EQUAL(p.type,"");
		BOOST_CHECK_EQUAL(p.value,"pi*7*6");
	}{
		try{
			P p("x foo bar");
		} catch(const Stencil::DirectiveException& exc){
			BOOST_CHECK_EQUAL(exc.type,"syntax");
		}
	}
}

BOOST_AUTO_TEST_CASE(includ){
	typedef Stencil::Include I;
	{
		I i("x");
		BOOST_CHECK_EQUAL(i.address.expr,"x");
		BOOST_CHECK_EQUAL(i.address.eval,false);
		BOOST_CHECK_EQUAL(i.select.expr,"");
		BOOST_CHECK_EQUAL(i.complete,false);
	}{
		I i("x select y");
		BOOST_CHECK_EQUAL(i.address.expr,"x");
		BOOST_CHECK_EQUAL(i.select.expr,"y");
		BOOST_CHECK_EQUAL(i.select.eval,false);
	}{
		I i(". select #id .class");
		BOOST_CHECK_EQUAL(i.address.expr,".");
		BOOST_CHECK_EQUAL(i.select.expr,"#id .class");
	}{
		I i("eval x+'stencil'");
		BOOST_CHECK_EQUAL(i.address.expr,"x+'stencil'");
		BOOST_CHECK_EQUAL(i.address.eval,true);
	}{
		I i("eval 'address'+'/'+'of/stencil' select eval '#macro-id'");
		BOOST_CHECK_EQUAL(i.address.expr,"'address'+'/'+'of/stencil'");
		BOOST_CHECK_EQUAL(i.address.eval,true);
		BOOST_CHECK_EQUAL(i.select.expr,"'#macro-id'");
		BOOST_CHECK_EQUAL(i.select.eval,true);
	}{
		I i("x complete");
		BOOST_CHECK_EQUAL(i.address.expr,"x");
		BOOST_CHECK_EQUAL(i.address.eval,false);
		BOOST_CHECK_EQUAL(i.complete,true);
	}
}

BOOST_AUTO_TEST_CASE(macro){
	typedef Stencil::Macro M;
	{
		M m("x");
		BOOST_CHECK_EQUAL(m.name,"x");
	}{
		M m("x-y_z");
		BOOST_CHECK_EQUAL(m.name,"x-y_z");
	}{
		try{
			M m("x*foo.bar");
		} catch(const Stencil::DirectiveException& exc){
			BOOST_CHECK_EQUAL(exc.type,"syntax");
		}
	}
}

BOOST_AUTO_TEST_SUITE_END()
