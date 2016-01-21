#include <memory>

#include <boost/regex.hpp>
#include <boost/test/unit_test.hpp>

#include <stencila/sheet.hpp>
using namespace Stencila;

class TestSpread : public Spread {
 public:

 	std::string execute(const std::string& source) {
 		return "";
 	}

	std::string evaluate(const std::string& expression){
		return "";
	}

	std::string set(const std::string& id, const std::string& expression, const std::string& name = ""){
		std::string type = "string";
		std::string value;
		if(expression.find("error")!=std::string::npos) {
			value = "There was an error!";
			type = "error";
		} else {
			value = expression;
		}
		variables_[id] = value;
		if (name.length()) variables_[name] = value;
		return type + " " + value;
	}

	std::string get(const std::string& name){
		return variables_[name];
	}

	std::string clear(const std::string& id = "", const std::string& name = ""){
		if (id=="") {
			variables_.clear();
		} else {
			variables_.erase(id);
			if (name.length()) {
				variables_.erase(name);
			}
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

	void store(const std::string& name, const std::string& value) {
	}

	std::string retrieve(const std::string& name) {
		return "";
	}

 private:
 	std::map<std::string,std::string> variables_;
};

BOOST_AUTO_TEST_SUITE(sheet_quick)

BOOST_AUTO_TEST_CASE(meta_attributes){
	Sheet s1;
	BOOST_CHECK_EQUAL(s1.title(),"");
	BOOST_CHECK_EQUAL(s1.description(),"");
	BOOST_CHECK_EQUAL(s1.authors().size(),0);
	BOOST_CHECK_EQUAL(s1.keywords().size(),0);

	Sheet s2;
	s2.attach(std::make_shared<TestSpread>());
	// Note that the TestSpread does not recognised quotes, so setting of these
	// attributes is a little different to normal (they are usually string expressions)
	s2.load(
		"title = A test sheet\n"
		"description = A sheet used for testing\n"
		"authors = Peter Pan, @captainhook\n"
		"keywords = data, is, gold"
	);
	s2.update();

	BOOST_CHECK_EQUAL(s2.title(),"A test sheet");
	BOOST_CHECK_EQUAL(s2.description(),"A sheet used for testing");
	
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
	Sheet::Cell cell;

	// Empty or blank (only whitespace) source is ignored
	BOOST_CHECK_EQUAL(Sheet::parse("").kind,'0');
	BOOST_CHECK_EQUAL(Sheet::parse("\t").kind,'0');
	BOOST_CHECK_EQUAL(Sheet::parse(" \t\n\t").kind,'0');

	// Tabs are replaced with spaces
	BOOST_CHECK_EQUAL(Sheet::parse("\t'foo\t\tbar'\t").expression,"'foo  bar'");

	// Spaces are insignificant at ends of expressions...
	BOOST_CHECK_EQUAL(Sheet::parse("42").expression,"42");
	BOOST_CHECK_EQUAL(Sheet::parse(" 42").expression,"42");
	BOOST_CHECK_EQUAL(Sheet::parse(" 'foo bar' ").expression,"'foo bar'");
	// ... but not for implicit strings
	BOOST_CHECK_EQUAL(Sheet::parse(" foo bar ").expression,"\" foo bar \"");

	// Named expressions
	for(auto content : {"answer = 6*7"," answer =6*7"," answer= 6*7 ","answer=6*7"}){
		cell = Sheet::parse(content);
		BOOST_CHECK_EQUAL(cell.kind,'1');
		BOOST_CHECK_EQUAL(cell.name,"answer");
		BOOST_CHECK_EQUAL(cell.expression,"6*7");
	}

	// Dynamic expressions
	cell = Sheet::parse("=42");
	BOOST_CHECK_EQUAL(cell.kind,'2');
	BOOST_CHECK_EQUAL(cell.expression,"42");
	BOOST_CHECK_EQUAL(cell.name,"");

	// Literal expressions
	cell = Sheet::parse("42");
	BOOST_CHECK_EQUAL(cell.kind,'n');
	BOOST_CHECK_EQUAL(cell.expression,"42");

	cell = Sheet::parse("3.14");
	BOOST_CHECK_EQUAL(cell.kind,'n');
	BOOST_CHECK_EQUAL(cell.expression,"3.14");

	cell = Sheet::parse("\"Double quoted string with an escaped double quote \\\" inside it\"");
	BOOST_CHECK_EQUAL(cell.kind,'s');
	BOOST_CHECK_EQUAL(cell.expression,"\"Double quoted string with an escaped double quote \\\" inside it\"");

	cell = Sheet::parse("\'Single quoted string with an escaped single quote \\\'' inside it\'");
	BOOST_CHECK_EQUAL(cell.kind,'s');
	BOOST_CHECK_EQUAL(cell.expression,"\'Single quoted string with an escaped single quote \\\'' inside it\'");

	cell = Sheet::parse("An implicit string");
	BOOST_CHECK_EQUAL(cell.kind,'z');
	BOOST_CHECK_EQUAL(cell.expression,"\"An implicit string\"");
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

BOOST_AUTO_TEST_CASE(dependencies_1){
	Sheet s;
	s.load(
		"= A2\t= A1     \t= C2 \n"
		"= C1\t= A1 + B1\t1\n"
	);
	s.attach(std::make_shared<TestSpread>());
	s.update();

	// Initial checks for loading
	BOOST_CHECK_EQUAL(join(s.list(), ","), "A1,A2,B1,B2,C1,C2");
	BOOST_CHECK_EQUAL(s.content("A1"), "A2");
	BOOST_CHECK_EQUAL(s.content("B2"), "A1 + B1");
	BOOST_CHECK_EQUAL(s.content("C2"), "1");

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
	BOOST_CHECK_EQUAL(s.source("B2"), "= C2");
	BOOST_CHECK_EQUAL(join(s.depends("B2"), ","), "C2");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "C2,B2,C1,A2,A1,B1");

	// Create a circular dependency
	BOOST_CHECK_THROW(s.update("B2","= A1 + B2"),Exception);
}

BOOST_AUTO_TEST_CASE(dependencies_2){
	Sheet s;
	s.load(
		"0\t= A1\n"
		"0\t= A2\n"
	);
	s.attach(std::make_shared<TestSpread>());
	s.update();

	BOOST_CHECK_EQUAL(join(s.depends("A1"), ","), "");
	BOOST_CHECK_EQUAL(join(s.depends("A2"), ","), "");
	BOOST_CHECK_EQUAL(join(s.depends("B1"), ","), "A1");
	BOOST_CHECK_EQUAL(join(s.depends("B2"), ","), "A2");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "A2,B2,A1,B1");

	s.update("A1","0");
	BOOST_CHECK_EQUAL(join(s.depends("A1"), ","), "");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "A2,B2,A1,B1");

	s.update("B1","0");
	BOOST_CHECK_EQUAL(join(s.depends("B1"), ","), "");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "B1,A2,B2,A1");
}

BOOST_AUTO_TEST_CASE(request){
	Sheet s;
	s.load(
		"1\t= A1\n"
		"2\t= A2\n"
	);
	s.attach(std::make_shared<TestSpread>());
	s.update();

	BOOST_CHECK_EQUAL(join(s.depends("B1"), ","), "A1");

	BOOST_CHECK_EQUAL(
		s.request("PUT","update",R"([{"id":"A1","source":"2"}])"),
		R"([{"id":"A1","kind":"n","type":"string","value":"2"},{"id":"B1","kind":"2","type":"string","value":"A1"}])"
	);

	BOOST_CHECK_EQUAL(
		s.request("PUT","update",R"([{"id":"A1","source":"some error"}])"),
		R"([{"id":"A1","kind":"z","type":"error","value":"There was an error!"},{"id":"B1","kind":"2","type":"string","value":"A1"}])"
	);

	BOOST_CHECK_EQUAL(
		s.request("PUT","update",R"([{"id":"A1","source":""}])"),
		R"([{"id":"B1","kind":"2","type":"string","value":"A1"}])"
	);
}

BOOST_AUTO_TEST_SUITE_END()


BOOST_AUTO_TEST_SUITE(sheet_slow)

BOOST_AUTO_TEST_CASE(view){
	// Must be called to register classes
	// before serving will work
	Component::classes();

	Sheet s;
	s.load("Hello world\n");
	s.attach(std::make_shared<TestSpread>());
	s.update();
	s.view();

	BOOST_CHECK(s.held());

	sleep(30);
}

BOOST_AUTO_TEST_SUITE_END()
