#include <memory>

#include <boost/regex.hpp>
#include <boost/test/unit_test.hpp>

#include <stencila/sheet.hpp>

BOOST_AUTO_TEST_SUITE(sheet_quick)

using namespace Stencila;

class TestSpread : public Spread {
 public:

	std::string set(const std::string& id, const std::string& expression, const std::string& alias=""){
		variables_[id] = expression;
		if (alias.length()) variables_[alias] = expression;
		return expression;
	}

	std::string get(const std::string& name){
		return variables_[name];
	}

	std::string clear(const std::string& name){
		if (name=="") {
			variables_.clear();
		} else {
			variables_.erase(name);
		}
		return "";
	}

	std::string list(void){
		std::vector<std::string> names;
		for (auto iter : variables_) {
			names.push_back(iter.first);
		}
		return join(names, ",");
	}

	std::string collect(const std::vector<std::string>& cells){
		return "[" + join(cells, ",") + "]";
	}

	std::string depends(const std::string& expression){
	    std::vector<std::string> depends;
	    boost::regex regex("\\w+");
	    boost::sregex_token_iterator iter(expression.begin(), expression.end(), regex, 0);
	    boost::sregex_token_iterator end;
	    std::copy(iter, end, std::back_inserter(depends));
		return join(depends, ",");
	}

 private:
 	std::map<std::string,std::string> variables_;
};


BOOST_AUTO_TEST_CASE(meta_attributes){
	Sheet s1;
	BOOST_CHECK_EQUAL(s1.title(),"");
	BOOST_CHECK_EQUAL(s1.description(),"");
	BOOST_CHECK_EQUAL(s1.authors().size(),0);
	BOOST_CHECK_EQUAL(s1.keywords().size(),0);

	Sheet s2;
	s2.load(
		"title = A title\n"
		"description = A description\n"
		"authors = Peter Pan, @captainhook\n"
		"keywords = data, is, gold"
	);

	BOOST_CHECK_EQUAL(s2.title(),"A title");
	BOOST_CHECK_EQUAL(s2.description(),"A description");
	
	auto a = s2.authors();
	BOOST_CHECK_EQUAL(a.size(),2);
	BOOST_CHECK_EQUAL(a[0],"Peter Pan");
	BOOST_CHECK_EQUAL(a[1],"@captainhook");

	auto k = s2.keywords();
	BOOST_CHECK_EQUAL(k.size(),3);
	BOOST_CHECK_EQUAL(k[0],"data");
	BOOST_CHECK_EQUAL(k[1],"is");
	BOOST_CHECK_EQUAL(k[2],"gold");
}

BOOST_AUTO_TEST_CASE(identify){
	BOOST_CHECK_EQUAL(Sheet::identify(0,0),"A1");
	BOOST_CHECK_EQUAL(Sheet::identify(1,0),"A2");

	BOOST_CHECK_EQUAL(Sheet::identify(1,1),"B2");
	BOOST_CHECK_EQUAL(Sheet::identify(2,2),"C3");

	BOOST_CHECK_EQUAL(Sheet::identify(0,25),"Z1");
	BOOST_CHECK_EQUAL(Sheet::identify(0,26),"AA1");
	BOOST_CHECK_EQUAL(Sheet::identify(0,27),"AB1");
	BOOST_CHECK_EQUAL(Sheet::identify(0,28),"AC1");

	BOOST_CHECK_EQUAL(Sheet::identify(0,52),"BA1");
}

BOOST_AUTO_TEST_CASE(is_id){
	BOOST_CHECK(Sheet::is_id("A1"));
	BOOST_CHECK(Sheet::is_id("AZHGE136762"));

	BOOST_CHECK(not Sheet::is_id("a1"));
	BOOST_CHECK(not Sheet::is_id("1A"));
	BOOST_CHECK(not Sheet::is_id("A0"));
}

BOOST_AUTO_TEST_CASE(index_col){
	BOOST_CHECK_EQUAL(Sheet::index_col("A"),0);
	BOOST_CHECK_EQUAL(Sheet::index_col("B"),1);
	BOOST_CHECK_EQUAL(Sheet::index_col("AA"),26);
	BOOST_CHECK_EQUAL(Sheet::index_col("AB"),27);
}

BOOST_AUTO_TEST_CASE(interpolate){
	BOOST_CHECK_EQUAL(join(Sheet::interpolate("A","1","A","1"), ","),"A1");
	BOOST_CHECK_EQUAL(join(Sheet::interpolate("A","1","A","3"), ","),"A1,A2,A3");
	BOOST_CHECK_EQUAL(join(Sheet::interpolate("A","1","B","2"), ","),"A1,A2,B1,B2");
}

BOOST_AUTO_TEST_CASE(parse){
	auto p0 = Sheet::parse("");
	BOOST_CHECK_EQUAL(p0[0],"");
	BOOST_CHECK_EQUAL(p0[1],"");
	BOOST_CHECK_EQUAL(p0[2],"");

	// Tabs are replaced with spaces
	BOOST_CHECK_EQUAL(Sheet::parse("\tfoo\t\tbar\t")[0]," foo  bar ");

	// Spaces are significant before, after and within a constant
	BOOST_CHECK_EQUAL(Sheet::parse("42")[0],"42");
	BOOST_CHECK_EQUAL(Sheet::parse(" 42")[0]," 42");
	BOOST_CHECK_EQUAL(Sheet::parse(" foo bar ")[0]," foo bar ");

	// Expressions
	for(auto content : {"= 6*7"," =6*7"," = 6*7  "}){
		auto p = Sheet::parse(content);
		BOOST_CHECK_EQUAL(p[0],"");
		BOOST_CHECK_EQUAL(p[1],"6*7");
		BOOST_CHECK_EQUAL(p[2],"");
	}

	// Expression with alias
	for(auto content : {"answer = 6*7"," answer =6*7"," answer= 6*7 "}){
		auto p = Sheet::parse(content);
		BOOST_CHECK_EQUAL(p[0],"");
		BOOST_CHECK_EQUAL(p[1],"6*7");
		BOOST_CHECK_EQUAL(p[2],"answer");
	}
}

BOOST_AUTO_TEST_CASE(translate){
	Sheet s;
	s.attach(std::make_shared<TestSpread>());

	BOOST_CHECK_EQUAL(s.translate("A1"),"A1");
	BOOST_CHECK_EQUAL(s.translate("A1:A3"),"[A1,A2,A3]");

	// Cell unions not yet implemented
	BOOST_CHECK_THROW(s.translate("A1&A2"),Exception); //"[A1,A2]"
	BOOST_CHECK_THROW(s.translate("A1:B2&C3"),Exception); //"[A1,A2,B1,B2,C3]"

	BOOST_CHECK_EQUAL(s.translate("func(A1:A3,A4)"),"func([A1,A2,A3],A4)");
}

BOOST_AUTO_TEST_CASE(dependency_graph){
	Sheet s;
	s.load(
		"= A2\t= A1     \t= C2 \n"
		"= C1\t= A1 + B1\t1\n"
	);
	s.attach(std::make_shared<TestSpread>());
	s.update();

	// Initial checks for loading
	BOOST_CHECK_EQUAL(join(s.list(), ","), "A1,A2,B1,B2,C1,C2");
	BOOST_CHECK_EQUAL(s.value("A1"), "A2");
	BOOST_CHECK_EQUAL(s.value("B2"), "A1 + B1");
	BOOST_CHECK_EQUAL(s.value("C2"), "1");

	// Check dependency graph
	BOOST_CHECK_EQUAL(join(s.depends("B2"), ","), "A1,B1");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "C2,C1,A2,A1,B1,B2");
	
	BOOST_CHECK_EQUAL(join(s.predecessors("A2"), ","), "C2,C1");
	BOOST_CHECK_EQUAL(s.predecessors("C2").size(),0);
	BOOST_CHECK_EQUAL(s.predecessors("foo").size(),0);

	BOOST_CHECK_EQUAL(join(s.successors("B1"), ","), "B2");
	BOOST_CHECK_EQUAL(s.successors("B2").size(),0);
	BOOST_CHECK_EQUAL(s.successors("foo").size(),0);

	// Change a cell
	s.update("B2","= C2");
	BOOST_CHECK_EQUAL(s.value("B2"), "C2");
	BOOST_CHECK_EQUAL(join(s.depends("B2"), ","), "C2");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "C2,B2,C1,A2,A1,B1");

	// Create a circular dependency
	BOOST_CHECK_THROW(s.update("B2","= A1 + B2"),Exception);
}

BOOST_AUTO_TEST_SUITE_END()
