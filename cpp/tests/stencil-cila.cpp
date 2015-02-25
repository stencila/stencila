#include <iostream>
 
#include <boost/test/unit_test.hpp>

#define STENCILA_CILA_PARSER_TRACE
#include <stencila/stencil-cila.hpp>  

using namespace Stencila;
struct CilaFixture : public CilaParser, public CilaGenerator {  
	Stencil stencil;

	// Methods added for debugging purposes
	
	void states_show(void){
		std::cout<<"-----------------States-------------------\n";
		for(auto state : states) std::cout<<state_name(state)<<"\n";
		std::cout<<"-----------------------------------------\n";
	}

	void nodes_show(void){
		std::cout<<"-----------------Nodes-------------------\n";
		for(auto node : nodes) std::cout<<node.indent.length()<<"\t"<<node.node.name()<<"\n";
		std::cout<<"-----------------------------------------\n";
	}

	void xml_show(void){
		std::cout<<"-------------------XML-------------------\n"
				<<stencil.xml()<<"\n"
				<<"-----------------------------------------\n";
	}
};

// Check macros. Macros are used so that Boost::Unit reports lines number
// of failed checks properly
#define CILA_XML(_CILA,_XML) { parse(stencil,_CILA); BOOST_CHECK_EQUAL(stencil.xml(),_XML); }
#define XML_CILA(_XML,_CILA) { stencil.xml(_XML); BOOST_CHECK_EQUAL(generate(stencil),_CILA); }
#define CILA_CILA(_IN,_OUT) { parse(stencil,_IN); BOOST_CHECK_EQUAL(generate(stencil),_OUT); }
#define ECHO(_IN) CILA_CILA(_IN,_IN);

BOOST_FIXTURE_TEST_SUITE(stencil_cila_quick,CilaFixture)
 
BOOST_AUTO_TEST_CASE(elements){
	CILA_XML("div","<div />");
	CILA_XML("div\ndiv","<div /><div />");
	CILA_XML("div\na\np","<div /><a /><p />");

	XML_CILA("<div />","div");
	XML_CILA("<div /><div />","div\ndiv");
	XML_CILA("<div /><a /><p />","div\na\np");

	ECHO("div\ntable\np\na\nhr");
}

BOOST_AUTO_TEST_CASE(empty){
	// Empty lines should be ignored
	ECHO("");
	CILA_CILA("\n","");
	CILA_CILA("div\n\ndiv","div\ndiv");
	CILA_CILA("div\n\ndiv\n\n\ndiv","div\ndiv\ndiv");
}

BOOST_AUTO_TEST_CASE(indentation){
	CILA_XML("div\ndiv","<div /><div />");
	CILA_XML("div\n\tp\n\t\ta\ndiv","<div><p><a /></p></div><div />");
	// Blank lines should not muck up indentation
	CILA_XML("div\n\n\tp\n\t\n  \n\n\tp\n\n \n\t\t\ta","<div><p /><p><a /></p></div>");

	XML_CILA("<div /><div><div /><div /></div><div />","div\ndiv\n\tdiv\n\tdiv\ndiv");
	XML_CILA("<div><div><div /></div></div><div />","div\n\tdiv\n\t\tdiv\ndiv");

	ECHO("div\n\tdiv\n\t\tdiv\n\t\t\tdiv\n\t\t\tdiv\n\t\tdiv");
}

BOOST_AUTO_TEST_CASE(auto_paragraphs){
	CILA_XML("No para","No para");
	CILA_XML("\nPara","<p>Para</p>");
	CILA_XML("\n\nPara","<p>Para</p>");
	CILA_XML("div\n\n\tPara1\n\t\n\tPara2\n\t\t\tPara2cont","<div><p>Para1</p><p>Para2Para2cont</p></div>");

	// Paragraph (empty line before)
	CILA_XML("div\n\nThis should be a paragraph","<div /><p>This should be a paragraph</p>")
	// Text (no empty line before)
	CILA_XML("This should NOT be a paragraph","This should NOT be a paragraph")
	CILA_XML("div\nThis should NOT be a paragraph","<div />This should NOT be a paragraph")
	// Nested text (no empty line before)
	CILA_XML("div\n\tThis should NOT be a paragraph","<div>This should NOT be a paragraph</div>")
	// Nested paragraph
	CILA_XML("div\n\n\tThis should be a paragraph","<div><p>This should be a paragraph</p></div>")

	XML_CILA("<p>Para</p>","\nPara");
	XML_CILA("<div><p>Para1</p><p>Para2</p></div>","div\n\t\n\tPara1\n\t\n\tPara2");

	ECHO("\nPara");
	ECHO("\nPara1\n\nPara2");
}

BOOST_AUTO_TEST_CASE(embedded){
	CILA_XML("div{div{div}}","<div><div><div /></div></div>");
	CILA_XML("div [id=yo] Some text {a [href=none] nowhere} after",R"(<div id="yo">Some text <a href="none">nowhere</a> after</div>)");
	CILA_XML("{div{div apple}{div pear}}",R"(<div><div>apple</div><div>pear</div></div>)");

	CILA_XML("Text with a no inlines","Text with a no inlines");
	CILA_XML("Text with a {a [href=http://stencil.la] link} in it.","Text with a <a href=\"http://stencil.la\">link</a> in it.");

	CILA_XML("{div}","<div />");
	CILA_XML("{div {div}}","<div><div /></div>");

	CILA_XML(
		"The minimum is {if a<b {write a}}{else {write b}}",
		"The minimum is <div data-if=\"a&lt;b\"><span data-write=\"a\" /></div><div data-else=\"true\"><span data-write=\"b\" /></div>"
	);

	CILA_XML("div\n\tSome inline {write pi*2}","<div>Some inline <span data-write=\"pi*2\" /></div>");

	CILA_XML("div Some text","<div>Some text</div>");
	CILA_XML("div {Some text}","<div>Some text</div>");
	CILA_XML("div Text with a {span inside span}.","<div>Text with a <span>inside span</span>.</div>");

	// Embedded syntax can be used for inline elements but is not generated
	CILA_CILA("{ul #id-to-prevent-autolist-style-cila {li apple}{li pear}}","ul #id-to-prevent-autolist-style-cila\n\tli apple\n\tli pear");
}

BOOST_AUTO_TEST_CASE(attributes){
	CILA_XML("div [class=a]",R"(<div class="a" />)");
	CILA_XML("div [attr= an attribute with leading and embedded spaces]",R"(<div attr=" an attribute with leading and embedded spaces" />)");
	CILA_XML("div #an-id",R"(<div id="an-id" />)");
	CILA_XML("div .a-class",R"(<div class="a-class" />)");
	CILA_XML("a [href=http://google.com] #an-id .a-class",R"(<a href="http://google.com" id="an-id" class="a-class" />)");

	CILA_XML("[class=a]",R"(<div class="a" />)");
	CILA_XML("#an-id",R"(<div id="an-id" />)");
	CILA_XML(".a-class",R"(<div class="a-class" />)");
	CILA_XML("#an-id .a-class",R"(<div id="an-id" class="a-class" />)");

	XML_CILA(R"(<li id="an-id" />)","li #an-id");
	XML_CILA(R"(<ul class="a-class" />)","ul .a-class");
	XML_CILA(R"(<a href="http://google.com" id="an-id" class="a-class" />)","a [href=http://google.com] #an-id .a-class");

	XML_CILA(R"(<div id="an-id" />)","#an-id");
	XML_CILA(R"(<div class="a-class" />)",".a-class");
	XML_CILA(R"(<div id="an-id" class="a-class" />)","#an-id .a-class");

	CILA_XML("a [href=http://stenci.la] Stencila","<a href=\"http://stenci.la\">Stencila</a>");
	ECHO("a [href=http://stenci.la] [title=Stencila] Stencila");
	// More than one
	CILA_XML("div [attr1=1] [attr2=2]","<div attr1=\"1\" attr2=\"2\" />");
	ECHO("ul [attr1=1] [attr2=2] [attr3=3]");
	// No need to include div
	CILA_XML("[attr=1]","<div attr=\"1\" />")
	ECHO("[attr=1]");

	ECHO("#an-id .a-class [href=google.com]");
	ECHO("li .a-class [href=google.com] #an-id");
}

BOOST_AUTO_TEST_CASE(id_class){
	// Shorthand CSS id and class works
	ECHO("ul #id");
	ECHO("ul .class");
	// Only one id
	CILA_CILA("ul #id1 #id2","ul #id2");
	// More than one class
	CILA_XML("div .klass","<div class=\"klass\" />");
	CILA_XML("div .klass1 .klass2","<div class=\"klass1 klass2\" />");
	CILA_XML("div .klass-a .klass-b .klass-c","<div class=\"klass-a klass-b klass-c\" />");
	// No need to include div
	ECHO("#id");
	CILA_XML(".class","<div class=\"class\" />");
	XML_CILA("<div class=\"class\" />",".class");
	ECHO(".class");
	// Mix them up
	ECHO("#id .class");
	// Multiple classes
	CILA_XML(".a .b .c #id","<div class=\"a b c\" id=\"id\" />");
	XML_CILA("<div class=\"a b c\" id=\"id\" />",".a .b .c #id");
	ECHO(".a .b .c .d");
}

BOOST_AUTO_TEST_CASE(meta){
	// Special IDs
	CILA_XML("#title My title","<div id=\"title\">My title</div>")
	CILA_XML("#description A short little stencil","<div id=\"description\">A short little stencil</div>")
	CILA_XML("#keywords foo,bar","<div id=\"keywords\">foo,bar</div>")
	CILA_XML(".author Joe Bloggs","<div class=\"author\">Joe Bloggs</div>")
	CILA_XML("#contexts r","<div id=\"contexts\">r</div>")
	CILA_XML("#theme beautiful","<div id=\"theme\">beautiful</div>")
}

BOOST_AUTO_TEST_CASE(exec){
	CILA_XML("r\n\ta=1","<pre data-exec=\"r\">\na=1\n</pre>");
	CILA_XML("r ~ &h34Ft7\n\ta=1","<pre data-exec=\"r\" data-hash=\"h34Ft7\">\na=1\n</pre>");

	XML_CILA("<pre data-exec=\"r\">a=1</pre>","r\n\ta=1");
	XML_CILA("<pre data-exec=\"r\">\na=1\n</pre>","r\n\ta=1");

	ECHO("r\n\ta=1");
}

BOOST_AUTO_TEST_CASE(exec_1){
	auto cila_ = 
R"(r
	pi <- 3.14)";
	auto html_ = 
R"(<pre data-exec="r">
pi &lt;- 3.14
</pre>)";
	CILA_XML(cila_,html_)
	ECHO(cila_)
}

BOOST_AUTO_TEST_CASE(exec_2){
	auto cila_ = 
R"(r
	pi <- 3.14
	print(pi))";
	auto html_ = 
R"(<pre data-exec="r">
pi &lt;- 3.14
print(pi)
</pre>)"; 
	CILA_XML(cila_,html_)
	ECHO(cila_)
}

BOOST_AUTO_TEST_CASE(exec_3){
	auto cila_ = 
R"(r
	pi <- 3.14
	2*pi
	2*pi*r^2
div
div)";
	auto html_ = 
R"(<pre data-exec="r">
pi &lt;- 3.14
2*pi
2*pi*r^2
</pre><div /><div />)";
	CILA_XML(cila_,html_)
	ECHO(cila_)
}

BOOST_AUTO_TEST_CASE(exec_with_empty_lines){
	auto cila_ = 
R"(r
	pi <- 3.14

	2*pi

	2*pi*r^2
div)";
	auto html_ = 
R"(<pre data-exec="r">
pi &lt;- 3.14

2*pi

2*pi*r^2
</pre><div />)";
	CILA_XML(cila_,html_); 
}

BOOST_AUTO_TEST_CASE(exec_with_empty_line_before_next){
	auto cila_ = 
R"(r
	pi <- 3.14
	
div)";
	auto html_ = 
R"(<pre data-exec="r">
pi &lt;- 3.14

</pre><div />)";
	CILA_XML(cila_,html_)
}

BOOST_AUTO_TEST_CASE(exec_with_text_after){
	auto cila_ = 
R"(r
	pi <- 3.14
Hello world)";
	auto html_ = 
R"(<pre data-exec="r">
pi &lt;- 3.14
</pre>Hello world)";
	CILA_XML(cila_,html_)
}

BOOST_AUTO_TEST_CASE(exec_image){
	auto cila_ = 
R"(r format png size 60x42px
	plot(x,y))";
	auto html_ = 
R"(<pre data-exec="r format png size 60x42px">
plot(x,y)
</pre>)";
	CILA_XML(cila_,html_)
	ECHO(cila_)
}

BOOST_AUTO_TEST_CASE(exec_with_lt){
	auto cila_ = 
R"(exec
	a<b
	a <b
	a< b
	a < b)";
	auto html_ = 
R"(<pre data-exec="exec">
a&lt;b
a &lt;b
a&lt; b
a &lt; b
</pre>)";
	CILA_XML(cila_,html_)
	ECHO(cila_)
}

BOOST_AUTO_TEST_CASE(directive_no_arg){
	CILA_XML("div else",R"(<div data-else="true" />)");
	CILA_XML("else",R"(<div data-else="true" />)");
	CILA_XML("div default",R"(<div data-default="true" />)");
	CILA_XML("default",R"(<div data-default="true" />)");

	XML_CILA(R"(<li data-else="true" />)","li else");
	XML_CILA(R"(<div data-else="true" />)","else");
	XML_CILA(R"(<li data-default="true" />)","li default");
	XML_CILA(R"(<div data-default="true" />)","default");

	ECHO("else");
	ECHO("li else");
	ECHO("default");
	ECHO("li default");
}

BOOST_AUTO_TEST_CASE(directive_arg){
	CILA_XML("div write x",R"(<div data-write="x" />)");
	CILA_XML("write x",R"(<span data-write="x" />)");
	CILA_XML("div if x",R"(<div data-if="x" />)");
	CILA_XML("if x",R"(<div data-if="x" />)");

	XML_CILA(R"(<div data-write="x" />)","div write x");
	XML_CILA(R"(<span data-write="x" />)","``x``");
	XML_CILA(R"(<li data-if="x" />)","li if x");
	XML_CILA(R"(<div data-if="x" />)","if x");

	ECHO("div write x");
	CILA_CILA("write x","``x``");
	ECHO("ul #an-id .a-class with x");
	ECHO("#an-id .a-class with x");
	CILA_CILA("div if x","if x");
	ECHO("if x");
}

BOOST_AUTO_TEST_CASE(flags){
	CILA_XML("~ &tH4dFg","<div data-hash=\"tH4dFg\" />");
	ECHO("~ &tH4dFg");

	CILA_XML("~ off","<div data-off=\"true\" />");
	ECHO("~ off");

	CILA_XML("~ ^42","<div data-index=\"42\" />");
	ECHO("~ ^42");

	CILA_XML("~ lock","<div data-lock=\"true\" />");
	ECHO("~ lock");

	CILA_XML("~ output","<div data-output=\"true\" />");
	ECHO("~ output");

	CILA_XML("~ included","<div data-included=\"true\" />");
	ECHO("~ included");

	CILA_XML("if x<0 ~ off",R"(<div data-if="x&lt;0" data-off="true" />)");
	ECHO("if x<0 ~ off");

	CILA_XML("write x ~ lock",R"(<span data-write="x" data-lock="true" />)");
	ECHO("write x ~ lock");

	ECHO("~ &tH4dFg off ^42 lock output");
	ECHO("p ~ &tH4dFg off ^42 lock output");
	ECHO("#id .class ~ &tH4dFg off ^42 lock output");
}

BOOST_AUTO_TEST_CASE(error){
	CILA_XML("~ !syntax","<div data-error=\"syntax\" />");
	CILA_XML("~ !exception(foo bar)","<div data-error=\"exception(foo bar)\" />");

	XML_CILA("<div data-error=\"syntax\" />","~ !syntax");
	XML_CILA("<div data-error=\"exception(foo bar)\" />","~ !exception(foo bar)");
}

BOOST_AUTO_TEST_CASE(directive_write){
	CILA_XML("write variable","<span data-write=\"variable\" />");
	CILA_XML("span write variable","<span data-write=\"variable\" />");
}

BOOST_AUTO_TEST_CASE(directive_with){
	CILA_XML("with what","<div data-with=\"what\" />")
	XML_CILA("<div data-with=\"what\" />","with what")
	ECHO("with what")

	ECHO("section with what")
}

BOOST_AUTO_TEST_CASE(directive_if){
	CILA_XML("if x<0\nelif x<1\nelse",R"(<div data-if="x&lt;0" /><div data-elif="x&lt;1" /><div data-else="true" />)");

	XML_CILA(R"(<div data-if="x&lt;0" /><div data-elif="x&lt;1" /><div data-else="true" />)","if x<0\nelif x<1\nelse");

	ECHO("if x<0\n\tA\nelif x<1\n\tB\nelse\n\tC");

	CILA_XML(
		"if true\n\tp .a\nelif false\n\tp .b\nelse\n\tp .c",
		"<div data-if=\"true\"><p class=\"a\" /></div><div data-elif=\"false\"><p class=\"b\" /></div><div data-else=\"true\"><p class=\"c\" /></div>"
	);
}

BOOST_AUTO_TEST_CASE(directive_switch){
	auto cila_ =
R"(switch a
	case 3.14
		Pi
	case 42
		The answer
	default
		A number)";
	auto html_ =
"<div data-switch=\"a\">"
	"<div data-case=\"3.14\">Pi</div>"
	"<div data-case=\"42\">The answer</div>"
	"<div data-default=\"true\">A number</div>"
"</div>";
	CILA_XML(cila_,html_);
	ECHO(cila_);
}

BOOST_AUTO_TEST_CASE(directive_for){
	ECHO("for item in items");
	CILA_XML("for item in items","<div data-for=\"item in items\" />");

	ECHO("for item in items\n\tp");
}

BOOST_AUTO_TEST_CASE(directive_include){
	ECHO("include address")
	CILA_XML("include address","<div data-include=\"address\" />")

	ECHO("include address selector")

	ECHO("include a-superbly-sublime-stencil #a-marvelous-macro")
	ECHO("include a-stencil-with-no-macro-defined .class-a [attr=\"x\"] .class-b")

	// Special '.' identifier for current stencil
	ECHO("macro hello\n\t``who``\ninclude . select #hello\n\tset who to 'world'")

	// Set directive
	ECHO("include stencil select selector\n\tset a to 4\n\tset b to 1")
}

BOOST_AUTO_TEST_CASE(modifiers){
	for(std::string modifier : {
		"delete",
		"replace",
		"change",
		"before",
		"after",
		"prepend",
		"append"
	}){
		ECHO(modifier+" selector")
		CILA_XML(modifier+" selector","<div data-"+modifier+"=\"selector\" />")
	}
}

BOOST_AUTO_TEST_CASE(directive_macro){
	CILA_XML("macro name\n\tpar x","<div data-macro=\"name\"><div data-par=\"x\" /></div>")

	ECHO("macro name")
}

BOOST_AUTO_TEST_CASE(directive_par){
	CILA_XML("par x","<div data-par=\"x\" />");
	CILA_XML("par x type text","<div data-par=\"x type text\" />");
	CILA_XML("par x type number value 42","<div data-par=\"x type number value 42\" />");
	CILA_XML("par x value \"a\"","<div data-par=\"x value &quot;a&quot;\" />");

	XML_CILA("<div data-par=\"x value &quot;a&quot;\" />","par x value \"a\"");
	
	ECHO("par x");
	ECHO("par x value 1");
	ECHO("par x type number value 42");
	ECHO("par x type text value \"Hello world\"");
}

BOOST_AUTO_TEST_CASE(sections){
	CILA_XML("> Heading",R"(<section id="heading"><h1>Heading</h1></section>)");
	CILA_XML("> Heading with spaces",R"(<section id="heading-with-spaces"><h1>Heading with spaces</h1></section>)");

	XML_CILA(R"(<section id="heading"><h1>Heading</h1></section>)","> Heading");
	XML_CILA(R"(<section id="heading-with-spaces"><h1>Heading with spaces</h1></section>)","> Heading with spaces");
	// Xml which does not convert to an autosection
	XML_CILA(R"(<section id="id-different-to-heading"><h1>Heading</h1></section>)","section #id-different-to-heading\n\th1 Heading");
	XML_CILA(R"(<section><p></p><h1>Heading not the first child</h1></section>)","section\n\tp\n\th1 Heading not the first child");

	ECHO("> Heading");
	ECHO("> Heading with spaces");
}

BOOST_AUTO_TEST_CASE(ul){
	CILA_XML("- apple\n- pear",R"(<ul><li>apple</li><li>pear</li></ul>)");
	CILA_XML("-apple\n-pear",R"(<ul><li>apple</li><li>pear</li></ul>)");
	CILA_XML("{-apple}{-pear}",R"(<ul><li>apple</li><li>pear</li></ul>)");
	// List items can have normal text parsing
	CILA_XML("- Some _emphasis_",R"(<ul><li>Some <em>emphasis</em></li></ul>)");
	CILA_XML("- An interpolated ``value``",R"(<ul><li>An interpolated <span data-write="value" /></li></ul>)");
	CILA_XML("- A link to [Google](http://google.com)",R"(<ul><li>A link to <a href="http://google.com">Google</a></li></ul>)");

	XML_CILA(R"(<ul><li>apple</li><li>pear</li></ul>)","- apple\n- pear");
	XML_CILA(R"(<ul><li>A link to <a href="http://google.com">Google</a></li></ul>)","- A link to [Google](http://google.com)");

	ECHO("- apple\n- pear");
	ECHO("- An interpolated ``value``\n- A bit of |math|\n- A bit of `code` too");
	
	ECHO("div\n\t- Should\n\t- be\n\t- indented\ndiv");
	ECHO("div\n\tdiv\n\t\t- Should\n\t\t- be\n\t\t- indented more");

	// <ul> with attributes or no <li> children are not autoed
	CILA_CILA("ul","ul");
	CILA_CILA("ul #an-id\n\ta","ul #an-id\n\ta");
	CILA_CILA("ul\n\ta","ul\n\ta");
}

BOOST_AUTO_TEST_CASE(ol){
	CILA_XML("1. apple\n2. pear",R"(<ol><li>apple</li><li>pear</li></ol>)");
	CILA_XML("1.apple\n2.pear",R"(<ol><li>apple</li><li>pear</li></ol>)");

	XML_CILA(R"(<ol><li>apple</li><li>pear</li></ol>)","1. apple\n2. pear");
	XML_CILA(R"(<ol id="an-id"><li>apple</li><li>pear</li></ol>)","ol #an-id\n\tli apple\n\tli pear");

	ECHO("1. apple\n2. pear\n3. apricot");
}

BOOST_AUTO_TEST_CASE(trailing_text){
	CILA_XML("div Hello",R"(<div>Hello</div>)");
	CILA_XML("a [href=http://google.com] Google",R"(<a href="http://google.com">Google</a>)");
	CILA_XML("div Some text with bits like #id and .class",R"(<div>Some text with bits like #id and .class</div>)");
	CILA_XML(".a-class else",R"(<div class="a-class" data-else="true" />)");

	CILA_XML("a my link","<a>my link</a>")
	CILA_XML("a [href=http://google.com] #id my link","<a href=\"http://google.com\" id=\"id\">my link</a>")
	
	//Space before trailing text is stripped
	CILA_XML("span foo","<span>foo</span>");
	CILA_XML("span            foo","<span>foo</span>");

	XML_CILA("<div>Short text only child trails</div><div />","div Short text only child trails\ndiv");
	XML_CILA(
		"<div>Long text only child is on next line and indented xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx</div>",
		"div\n\tLong text only child is on next line and indented xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
	);
	XML_CILA("<div>Text with block siblings does not trail<div/></div>","div\n\tText with block siblings does not trail\n\tdiv");

	ECHO("div Hello");
	ECHO("div Some text with bits like #id and .class");
	ECHO(".a-class else");
}

BOOST_AUTO_TEST_CASE(text){
	CILA_XML("","");
	CILA_XML("Hello world","Hello world");

	XML_CILA("","");
	XML_CILA("Hello world","Hello world");

	ECHO("Hello world");
}

BOOST_AUTO_TEST_CASE(emphasis){
	CILA_XML("Is _emphasised_","Is <em>emphasised</em>");
	CILA_XML("Some _emphasised_ text","Some <em>emphasised</em> text");
	CILA_XML("This is _emphasised_. But this is not.","This is <em>emphasised</em>. But this is not.");
	CILA_XML("not_emphasised","not_emphasised");
	CILA_XML("not_emphasised_ text","not_emphasised_ text");

	XML_CILA("<em>emphasised</em>","_emphasised_");
	//!XML_CILA("Some <em>emphasised</em> text","Some _emphasised_ text");

	ECHO("_emphasised_");
}

BOOST_AUTO_TEST_CASE(strong){
	CILA_XML("Is *strong*","Is <strong>strong</strong>");
	CILA_XML("Some *strong* text","Some <strong>strong</strong> text");
	CILA_XML("This is *strong*. But this is not.","This is <strong>strong</strong>. But this is not.");
	CILA_XML("not*strong","not*strong");
	CILA_XML("some not*strong* text","some not*strong* text");

	XML_CILA("<strong>strong</strong>","*strong*");
	//!XML_CILA("Some <strong>strong</strong> text","Some *strong* text");
	
	ECHO("*strong*");
}

BOOST_AUTO_TEST_CASE(emphasis_strong){
	CILA_XML("Some _emphasised *strong* text_","Some <em>emphasised <strong>strong</strong> text</em>");
	CILA_XML("Some *strong _emphasised_ text*","Some <strong>strong <em>emphasised</em> text</strong>");

	//!XML_CILA("Some <em>emphasised <strong>strong</strong> text</em>","Some _emphasised *strong* text_");
	//!XML_CILA("Some <strong>strong <em>emphasised</em> text</strong>","Some *strong _emphasised_ text*");
	
	//! ECHO("Some _emphasised *strong* text_");
	//! ECHO("Some *strong _emphasised_ text*")
}

BOOST_AUTO_TEST_CASE(code){
	CILA_XML("`e=mc^2`","<code>e=mc^2</code>");
	CILA_XML("An escaped backtick \\` within text","An escaped backtick ` within text");
	CILA_XML("An escaped backtick within code `\\``","An escaped backtick within code <code>`</code>");

	XML_CILA("<code>e=mc^2</code>","`e=mc^2`");
	XML_CILA("An escaped backtick ` within text","An escaped backtick \\` within text");
	//!XML_CILA("An escaped backtick within code <code>`</code>","An escaped backtick within code `\\``");
	
	ECHO("`e=mc^2`");
	//! ECHO("Before `e=mc^2` after");
	ECHO("An escaped backtick \\` within text");
}

BOOST_AUTO_TEST_CASE(asciimath){
	CILA_XML("|e=mc^2|",R"(<p class="equation"><script type="math/asciimath; mode=display">e=mc^2</script></p>)");
	CILA_XML("Text before |e=mc^2|",R"(Text before <script type="math/asciimath">e=mc^2</script>)");
	CILA_XML("Text before |e=mc^2| text after",R"(Text before <script type="math/asciimath">e=mc^2</script> text after)");
	CILA_XML("With asterisks and underscores |a_b*c|",R"(With asterisks and underscores <script type="math/asciimath">a_b*c</script>)");
	CILA_XML("An escaped pipe within AsciiMath |a\\|b|",R"(An escaped pipe within AsciiMath <script type="math/asciimath">a|b</script>)");

	//!XML_CILA(R"(Before <script type="math/asciimath">e=mc^2</script> after)","Before |e=mc^2| after");
	XML_CILA(R"(<p class="equation"><script type="math/asciimath; mode=display">e=mc^2</script></p>)","|e=mc^2|");
	XML_CILA(R"(An escaped pipe | within text)","An escaped pipe \\| within text");
	//!XML_CILA(R"(An escaped pipe within AsciiMath <span class="math"><script type="math/asciimath">a|b</script></span>)","An escaped pipe within AsciiMath |a\\|b|");

	ECHO("|e=mc^2|");
	//! ECHO("Before |e=mc^2| after");
	//! ECHO("An escaped pipe within AsciiMath |a\\|b|");
	ECHO("An escaped pipe \\| within text)");
}

BOOST_AUTO_TEST_CASE(tex){
	CILA_XML("\\(e=mc^2\\)",R"(<p class="equation"><script type="math/tex; mode=display">e=mc^2</script></p>)");

	XML_CILA(R"(<p class="equation"><script type="math/tex; mode=display">e=mc^2</script></p>)","\\(e=mc^2\\)");

	ECHO("\\(e=mc^2\\)");
	//! ECHO("Before \\(e=mc^2\\) after");
}

BOOST_AUTO_TEST_CASE(link){
	CILA_XML("[t-test](http://en.wikipedia.org/wiki/Student's_t-test)",R"(<a href="http://en.wikipedia.org/wiki/Student's_t-test">t-test</a>)");
	CILA_XML("Go to [Google](http://google.com)",R"(Go to <a href="http://google.com">Google</a>)");
	CILA_XML("[Google](http://google.com) is a link",R"(<a href="http://google.com">Google</a> is a link)");

	XML_CILA(R"(<a href="http://en.wikipedia.org/wiki/Student's_t-test">t-test</a>)","[t-test](http://en.wikipedia.org/wiki/Student's_t-test)");
	//!XML_CILA(R"(Go to <a href="http://google.com">Google</a>)","Go to [Google](http://google.com)");
	//!XML_CILA(R"(<a href="http://google.com">Google</a> is a link)","[Google](http://google.com) is a link");
	
	ECHO("[t-test](http://en.wikipedia.org/wiki/Student's_t-test)");
	//! ECHO("Before [t-test](http://en.wikipedia.org/wiki/Student's_t-test) after");
}

BOOST_AUTO_TEST_CASE(autolink){
	CILA_XML("http://google.com",R"(<a href="http://google.com">http://google.com</a>)");
	CILA_XML("Go to https://google.com",R"(Go to <a href="https://google.com">https://google.com</a>)");
	CILA_XML("An autolink http://google.com with text after it",R"(An autolink <a href="http://google.com">http://google.com</a> with text after it)");

	XML_CILA(R"(<a href="http://google.com">http://google.com</a>)","http://google.com");
	//!XML_CILA(R"(Go to <a href="https://google.com">https://google.com</a>)","Go to https://google.com");
	//!XML_CILA(R"(An autolink <a href="http://google.com">http://google.com</a> with text after it)","An autolink http://google.com with text after it");
	
	ECHO("http://google.com");
	ECHO("https://google.com");
	//!ECHO("Before http://google.com after");
}

BOOST_AUTO_TEST_CASE(autoemail){
	CILA_XML("someone@example.com","<a href=\"mailto:someone@example.com\">someone@example.com</a>");
	XML_CILA("<a href=\"mailto:someone@example.com\">someone@example.com</a>","someone@example.com");
	ECHO("someone@example.com");
}

BOOST_AUTO_TEST_CASE(refer){
	CILA_XML("@figure-x-y",R"(<span data-refer="#figure-x-y" />)");
	CILA_XML("An escaped at \\@ in text","An escaped at @ in text");

	XML_CILA(R"(<span data-refer="#figure-x-y" />)","@figure-x-y");
	XML_CILA("An at @ in text","An at \\@ in text");

	CILA_XML("refer selector with space",R"(<span data-refer="selector with space" />)");
	XML_CILA(R"(<span data-refer="selector with space" />)","refer selector with space");

	ECHO("@figure-x-y");
	ECHO("refer section#intro figure");
	ECHO("\\@");
}

BOOST_AUTO_TEST_CASE(interpolate){
	CILA_XML("``x``",R"(<span data-write="x" />)");
	CILA_XML("The answer is ``6*7``!",R"(The answer is <span data-write="6*7" />!)");

	XML_CILA(R"(<span data-write="x" />)","``x``");
	//!XML_CILA(R"(The answer is <span data-write="6*7" />!)","The answer is ``6*7``!");
	
	ECHO("``x``");
	//!ECHO("Before ``x`` after");
}

BOOST_AUTO_TEST_SUITE_END()
