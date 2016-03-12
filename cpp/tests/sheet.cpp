#include <memory>

#include <boost/regex.hpp>
#include <boost/test/unit_test.hpp>

#include <stencila/sheet.hpp>
#include <stencila/function.hpp>
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

	std::vector<std::string> functions(void) {
		return {};
	}

	Function function(const std::string& name) {
		return Function();
	}

    void read(const std::string& path) {
    }

    void write(const std::string& path) {
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
	s2.cells({
		{"A1","title = A test sheet"},
		{"A2","description = A sheet used for testing"},
		{"A3","authors = Peter Pan, @captainhook"},
		{"A4","keywords = data, is, gold"}
	});

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
	typedef Sheet::Cell Cell;
	Cell cell;

	// Basic detection of kind (one of each kind, more variants below)
	#define CHECK(source_,kind_) \
		BOOST_CHECK_EQUAL(cell.source(source_).kind, kind_);

	CHECK("", Cell::blank_)
	CHECK("= 6*7", Cell::expression_)
	CHECK(": matrix", Cell::mapping_)
	CHECK("^ library(foo)", Cell::requirement_)
	CHECK("| optim()", Cell::manual_)
	CHECK("? some_test()==0", Cell::test_)
	CHECK("~ A1:C4 as points", Cell::visualization_)
	CHECK("_ some *Cila*", Cell::cila_)

	#undef CHECK


	// Optional name for all of the above
	#define CHECK(source_,kind_) \
		cell.source(source_); \
		BOOST_CHECK_EQUAL(cell.name, "foo"); \
		BOOST_CHECK_EQUAL(cell.expression, "42"); \
		BOOST_CHECK_EQUAL(cell.kind, kind_);

	CHECK("foo = 42", Cell::expression_)
	CHECK("foo : 42", Cell::mapping_)
	CHECK("foo ^ 42", Cell::requirement_)
	CHECK("foo | 42", Cell::manual_)
	CHECK("foo ? 42", Cell::test_)
	CHECK("foo ~ 42", Cell::visualization_)
	CHECK("foo _ 42", Cell::cila_)

	#undef CHECK

	// Empty or blank (only whitespace) source is ignored
	BOOST_CHECK_EQUAL(cell.source("").kind, Cell::blank_);
	BOOST_CHECK_EQUAL(cell.source("\t").kind, Cell::blank_);
	BOOST_CHECK_EQUAL(cell.source(" \t\n\t").kind, Cell::blank_);

	// Tabs are replaced with spaces
	BOOST_CHECK_EQUAL(cell.source("\t'foo\t\tbar'\t").expression,"'foo  bar'");

	// Spaces are insignificant at ends of expressions...
	BOOST_CHECK_EQUAL(cell.source("42").expression,"42");
	BOOST_CHECK_EQUAL(cell.source(" 42").expression,"42");
	BOOST_CHECK_EQUAL(cell.source(" 'foo bar' ").expression,"'foo bar'");
	// ... but not for implicit strings
	BOOST_CHECK_EQUAL(cell.source(" foo bar ").expression,"\" foo bar \"");

	// Named expressions
	for(auto content : {"answer = 6*7"," answer =6*7"," answer= 6*7 ","answer=6*7"}){
		cell.source(content);
		BOOST_CHECK_EQUAL(cell.kind, Cell::expression_);
		BOOST_CHECK_EQUAL(cell.name, "answer");
		BOOST_CHECK_EQUAL(cell.expression, "6*7");
	}

	// Dynamic expressions
	cell.source("=42");
	BOOST_CHECK_EQUAL(cell.kind, Cell::expression_);
	BOOST_CHECK_EQUAL(cell.expression,"42");
	BOOST_CHECK_EQUAL(cell.name,"");

	// Literal expressions
	cell.source("42");
	BOOST_CHECK_EQUAL(cell.kind, Cell::number_);
	BOOST_CHECK_EQUAL(cell.expression,"42");

	cell.source("3.14");
	BOOST_CHECK_EQUAL(cell.kind, Cell::number_);
	BOOST_CHECK_EQUAL(cell.expression,"3.14");

	cell.source("\"Double quoted string with an escaped double quote \\\" inside it\"");
	BOOST_CHECK_EQUAL(cell.kind, Cell::string_);
	BOOST_CHECK_EQUAL(cell.expression,"\"Double quoted string with an escaped double quote \\\" inside it\"");

	cell.source("\'Single quoted string with an escaped single quote \\\'' inside it\'");
	BOOST_CHECK_EQUAL(cell.kind, Cell::string_);
	BOOST_CHECK_EQUAL(cell.expression,"\'Single quoted string with an escaped single quote \\\'' inside it\'");

	cell.source("Some text");
	BOOST_CHECK_EQUAL(cell.kind, Cell::text_);
	BOOST_CHECK_EQUAL(cell.expression,"\"Some text\"");
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
	s.attach(std::make_shared<TestSpread>());
	s.cells({
		{"A1","= A2"},{"B1","= A1"},     {"C1","= C2"},
		{"A2","= C1"},{"B2","= A1 + B1"},{"C2","1"}
	});

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
	BOOST_CHECK_EQUAL(s.cell("B2").source(), "= C2");
	BOOST_CHECK_EQUAL(join(s.depends("B2"), ","), "C2");
	BOOST_CHECK_EQUAL(join(s.order(), ","), "C2,B2,C1,A2,A1,B1");

	// Create a circular dependency
	BOOST_CHECK_THROW(s.update("B2","= A1 + B2"),Exception);
}

BOOST_AUTO_TEST_CASE(dependencies_2){
	Sheet s;
	s.attach(std::make_shared<TestSpread>());
	s.cells({
		{"A1","0"},{"B1","= A1"},
		{"A2","0"},{"B2","= A2"},
	});

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

BOOST_AUTO_TEST_CASE(update){
	Sheet s;
	s.attach(std::make_shared<TestSpread>());

	std::vector<Sheet::Cell> changes;
	std::vector<Sheet::Cell> updates;

	updates = s.update("A1","1");

	BOOST_CHECK_EQUAL(updates.size(),1);
	if(updates.size()==1){
		auto cell = updates[0];
		BOOST_CHECK_EQUAL(cell.id, "A1");
		BOOST_CHECK_EQUAL(cell.kind_string(), "num");
		BOOST_CHECK_EQUAL(cell.value, "1");
	}

	updates = s.update("A2", "=A1");

	BOOST_CHECK_EQUAL(join(s.order(),","),"A1,A2");
	BOOST_CHECK_EQUAL(join(s.depends("A2"),","),"A1");
	BOOST_CHECK_EQUAL(updates.size(),1);
	if(updates.size()==1){
		auto cell = updates[0];
		BOOST_CHECK_EQUAL(cell.kind_string(), "exp");
	}

	updates = s.update("A3", "_ *bold*");

	BOOST_CHECK_EQUAL(updates.size(),1);
	if(updates.size()==1){
		auto cell = updates[0];
		BOOST_CHECK_EQUAL(cell.kind_string(), "cil");
		BOOST_CHECK_EQUAL(cell.type, "html");
		BOOST_CHECK_EQUAL(cell.value, "<p><strong>bold</strong></p>");
	}
}

BOOST_AUTO_TEST_CASE(request){
	Sheet s;
	s.attach(std::make_shared<TestSpread>());

	// Checking the response from update()
	#define CHECK(in_,out_) \
		BOOST_CHECK_EQUAL(s.request("PUT","update",in_), out_);

	CHECK(
		R"([[{"id":"A1","source":""}]])",
		R"([])"
	);
	
	CHECK(
		R"([[{"id":"A1","source":"2"}]])",
		R"([{"display":"cli","id":"A1","kind":"num","type":"string","value":"2"}])"
	);

	CHECK(
		R"([[{"id":"A1","source":"'string'"}]])",
		R"([{"display":"cli","id":"A1","kind":"str","type":"string","value":"'string'"}])"
	);

	CHECK(
		R"([[{"id":"A1","source":"= some error"}]])",
		R"([{"display":"exp","id":"A1","kind":"exp","type":"error","value":"There was an error!"}])"
	);

	CHECK(
		R"([[{"id":"A1","source":": matrix"}]])",
		R"([{"display":"cli","id":"A1","kind":"map","type":"string","value":"matrix"}])"
	);

	#undef CHECK
}

BOOST_AUTO_TEST_SUITE_END()


#if 0
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
#endif
