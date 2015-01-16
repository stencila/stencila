#include <iostream>
 
#include <boost/test/unit_test.hpp> 

#define STENCILA_CILA_PARSER_TRACE
#include <stencila/stencil-cila.hpp>  

//# indicates where a test is failing due to improper parsing

//! indicates where a test is failing due to improper generation of indentation
//! for inline elements

using namespace Stencila;
struct CilaFixture : public CilaParser, public CilaGenerator {  
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
#define CILA_XML(_CILA,_XML) BOOST_CHECK_EQUAL(parse(_CILA).stencil.xml(),_XML);
#define XML_CILA(_XML,_CILA) BOOST_CHECK_EQUAL(generate(_XML),_CILA);
#define CILA_CILA(_IN,_OUT) BOOST_CHECK_EQUAL(generate(parse(_IN).stencil.xml()),_OUT);
#define ECHO(_IN) CILA_CILA(_IN,_IN);

BOOST_FIXTURE_TEST_SUITE(cila,CilaFixture)
 
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

	XML_CILA("<p>Para</p>","\nPara");
	XML_CILA("<div><p>Para1</p><p>Para2</p></div>","div\n\t\n\tPara1\n\t\n\tPara2");

	ECHO("\nPara");
	ECHO("\nPara1\n\nPara2");
}

BOOST_AUTO_TEST_CASE(embedded){
	CILA_XML("div{div{div}}","<div><div><div /></div></div>");
	CILA_XML("div id=yo Some text {a href=none nowhere} after",R"(<div id="yo">Some text <a href="none">nowhere</a> after</div>)");
	CILA_XML("{div{div apple}{div pear}}",R"(<div><div>apple</div><div>pear</div></div>)");

	// Embedded elements are shortcuts for input and are not generated
	CILA_CILA("{ul #id-to-prevent-autolist-style-cila {li apple}{li pear}}","ul #id-to-prevent-autolist-style-cila\n\tli apple\n\tli pear");
}

BOOST_AUTO_TEST_CASE(attributes){
	CILA_XML("div class=a",R"(<div class="a" />)");
	CILA_XML("div #an-id",R"(<div id="an-id" />)");
	CILA_XML("div .a-class",R"(<div class="a-class" />)");
	CILA_XML("a href=http://google.com #an-id .a-class",R"(<a href="http://google.com" id="an-id" class="a-class" />)");

	CILA_XML("class=a",R"(<div class="a" />)");
	CILA_XML("#an-id",R"(<div id="an-id" />)");
	CILA_XML(".a-class",R"(<div class="a-class" />)");
	CILA_XML("#an-id .a-class",R"(<div id="an-id" class="a-class" />)");

	XML_CILA(R"(<li id="an-id" />)","li #an-id");
	XML_CILA(R"(<ul class="a-class" />)","ul .a-class");
	XML_CILA(R"(<a href="http://google.com" id="an-id" class="a-class" />)","a href=http://google.com #an-id .a-class");

	XML_CILA(R"(<div id="an-id" />)","#an-id");
	XML_CILA(R"(<div class="a-class" />)",".a-class");
	XML_CILA(R"(<div id="an-id" class="a-class" />)","#an-id .a-class");

	CILA_XML("a href=http://stenci.la Stencila","<a href=\"http://stenci.la\">Stencila</a>");
	ECHO("a href=http://stenci.la title=Stencila Stencila");
	// More than one
	CILA_XML("div attr1=1 attr2=2","<div attr1=\"1\" attr2=\"2\" />");
	ECHO("ul attr1=1 attr2=2 attr3=3");
	// No need to include div
	CILA_XML("attr=1","<div attr=\"1\" />")
	ECHO("attr=1");

	ECHO("#an-id .a-class href=google.com");
	ECHO("li .a-class href=google.com #an-id");
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

BOOST_AUTO_TEST_CASE(exec){
	//# CILA_XML("r\n\ta=1","<pre data-exec=\"r\">\na=1\n</pre>");

	XML_CILA("<pre data-exec=\"r\">a=1</pre>","r\n\ta=1");
	XML_CILA("<pre data-exec=\"r\">\na=1\n</pre>","r\n\ta=1");

	//# ECHO("r\n\ta=1");
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

BOOST_AUTO_TEST_CASE(if_elif_else){
	CILA_XML("if x<0\nelif x<1\nelse",R"(<div data-if="x&lt;0" /><div data-elif="x&lt;1" /><div data-else="true" />)");

	XML_CILA(R"(<div data-if="x&lt;0" /><div data-elif="x&lt;1" /><div data-else="true" />)","if x<0\nelif x<1\nelse");

	ECHO("if x<0\n\tA\nelif x<1\n\tB\nelse\n\tC");
}

BOOST_AUTO_TEST_CASE(trailing_text){
	CILA_XML("div Hello",R"(<div>Hello</div>)");
	CILA_XML("a href=http://google.com Google",R"(<a href="http://google.com">Google</a>)");
	CILA_XML("div Some text with bits like #id and .class",R"(<div>Some text with bits like #id and .class</div>)");
	CILA_XML(".a-class else",R"(<div class="a-class" data-else="true" />)");

	CILA_XML("a my link","<a>my link</a>")
	CILA_XML("a href=http://google.com #id my link","<a href=\"http://google.com\" id=\"id\">my link</a>")
	
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
	CILA_XML("_emphasised_","<em>emphasised</em>");
	CILA_XML("Some _emphasised_ text","Some <em>emphasised</em> text");

	XML_CILA("<em>emphasised</em>","_emphasised_");
	//!XML_CILA("Some <em>emphasised</em> text","Some _emphasised_ text");
	
	ECHO("_emphasised_");
}

BOOST_AUTO_TEST_CASE(strong){
	CILA_XML("*strong*","<strong>strong</strong>");
	CILA_XML("Some *strong* text","Some <strong>strong</strong> text");

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
	CILA_XML("|e=mc^2|",R"(<span class="math"><script type="math/asciimath">e=mc^2</script></span>)");
	CILA_XML("Text before |e=mc^2|",R"(Text before <span class="math"><script type="math/asciimath">e=mc^2</script></span>)");
	CILA_XML("|e=mc^2| text after",R"(<span class="math"><script type="math/asciimath">e=mc^2</script></span> text after)");
	CILA_XML("With asterisks and underscores |a_b*c|",R"(With asterisks and underscores <span class="math"><script type="math/asciimath">a_b*c</script></span>)");
	CILA_XML("An escaped pipe within AsciiMath |a\\|b|",R"(An escaped pipe within AsciiMath <span class="math"><script type="math/asciimath">a|b</script></span>)");

	XML_CILA(R"(<span class="math"><script type="math/asciimath">e=mc^2</script></span>)","|e=mc^2|");
	XML_CILA(R"(An escaped pipe | within text)","An escaped pipe \\| within text");
	//!XML_CILA(R"(An escaped pipe within AsciiMath <span class="math"><script type="math/asciimath">a|b</script></span>)","An escaped pipe within AsciiMath |a\\|b|");

	ECHO("|e=mc^2|");
	//! ECHO("Before |e=mc^2| after");
	//! ECHO("An escaped pipe within AsciiMath |a\\|b|");
	ECHO("An escaped pipe \\| within text)");
}

BOOST_AUTO_TEST_CASE(tex){
	CILA_XML("\\(e=mc^2\\)",R"(<span class="math"><script type="math/tex">e=mc^2</script></span>)");

	XML_CILA(R"(<span class="math"><script type="math/tex">e=mc^2</script></span>)","\\(e=mc^2\\)");

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

BOOST_AUTO_TEST_CASE(interpolate){
	CILA_XML("``x``",R"(<span data-write="x" />)");
	CILA_XML("The answer is ``6*7``!",R"(The answer is <span data-write="6*7" />!)");

	XML_CILA(R"(<span data-write="x" />)","``x``");
	//!XML_CILA(R"(The answer is <span data-write="6*7" />!)","The answer is ``6*7``!");
	
	ECHO("``x``");
	//!ECHO("Before ``x`` after");
}

BOOST_AUTO_TEST_SUITE_END()
