#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

BOOST_AUTO_TEST_SUITE(stencil_cila)

using namespace Stencila;

// Some checking functions and macros
// Macros are used so that Boost::Unit reports lines number
// of failed chaks properly

/**
 * Check Cila to HTML
 */
std::string html(const std::string& cila,bool indent=true){
    Stencil s;
    s.cila(cila);
    std::string html = s.html(false,indent);
    trim(html);
    return html;
}
#define HTML_(_CILA,_HTML) BOOST_CHECK_EQUAL(html(_CILA),_HTML);
#define HTML__(_CILA,_HTML) BOOST_CHECK_EQUAL(html(_CILA,false),_HTML);

/**
 * Check HTML to Cila
 */
std::string cila(const std::string& html){
    Stencil s;
    s.html(html);
    std::string cila = s.cila();
    trim(cila);
    return cila;
}
#define CILA_(_HTML,_CILA) BOOST_CHECK_EQUAL(cila(_HTML),_CILA);

/**
 * Check Cila to Cila
 */
std::string echo(const std::string& in){
    Stencil s;
    s.cila(in);
    std::string out = s.cila();
    trim(out);
    return out;
}
#define ECHO_(_CILA) BOOST_CHECK_EQUAL(echo(_CILA),_CILA);

// Check Cila to Cila, allowing for differences in input/output
#define BACK_(_IN,_OUT) BOOST_CHECK_EQUAL(echo(_IN),_OUT);


BOOST_AUTO_TEST_CASE(empty){
    // Empty lines should be ignored
    ECHO_("")
    BACK_("\n","")
    BACK_("div\n\ndiv","div\ndiv")
    BACK_("div\n\ndiv\n\n\ndiv","div\ndiv\ndiv")
}

BOOST_AUTO_TEST_CASE(indentation){
    ECHO_("ul\n\tli\n\tli")
    ECHO_("div\n\tdiv\n\t\tdiv")
    ECHO_("div\n\tdiv\n\t\tdiv\n\tdiv\ndiv")

    // Check that empty lines don't cause errors
    auto cila_1 = "#a\n\t#aa\n\t#ab\n\t\n\t#ac";
    auto cila_2 = "#a\n\t\t#aa\n\t\t#ab\n\t\t\n\t\t#ac";
    auto html_ = R"(<div id="a">
	<div id="aa" />
	<div id="ab" />
	<div id="ac" />
</div>)";
    HTML_(cila_1,html_);
    HTML_(cila_2,html_);
}

BOOST_AUTO_TEST_CASE(text){
    // Anything that is not a div should be plain text
    // but not on the first line (in that case it is assumed to be a paragraph)
    HTML_("div\nplain text","<div />\nplain text");
}

BOOST_AUTO_TEST_CASE(mono){
    HTML_("`code` at start","<code>code</code>\n at start");
    HTML_("In the `code` middle","In the \n<code>code</code>\n middle");
    HTML_("At end `code`","At end \n<code>code</code>");

    ECHO_("`code` at start")
    ECHO_("At end `code`")

}

BOOST_AUTO_TEST_CASE(math){
    HTML_("|E=mc^2| at start","<script type=\"math/asciimath\">E=mc^2</script>\n at start");
    HTML_("In the |E=mc^2| middle","In the \n<script type=\"math/asciimath\">E=mc^2</script>\n middle");
    HTML_("At end |E=mc^2|","At end \n<script type=\"math/asciimath\">E=mc^2</script>");

    ECHO_("|E=mc^2| at start")
    ECHO_("At end |E=mc^2|")
}

BOOST_AUTO_TEST_CASE(emphasis){
    HTML_("_word_","<em>word</em>");
    HTML_("*word*","<em>word</em>");

    ECHO_("_word_");
}

BOOST_AUTO_TEST_CASE(strong){
    HTML_("__word__","<strong>word</strong>");
    HTML_("**word**","<strong>word</strong>");

    ECHO_("__word__");
}

BOOST_AUTO_TEST_CASE(link){
    HTML_("[t-test](http://en.wikipedia.org/wiki/Student's_t-test)","<a href=\"http://en.wikipedia.org/wiki/Student's_t-test\">t-test</a>");
    
    ECHO_("[Google](http://google.com)")
    ECHO_("Go to [Google](http://google.com)")
    ECHO_("[Google](http://google.com) is a link")
}

BOOST_AUTO_TEST_CASE(different_numbers_of_inlines){
    HTML_("`code` and |math|","<code>code</code>\n and \n<script type=\"math/asciimath\">math</script>");
}

BOOST_AUTO_TEST_CASE(elements_with_trailing_text){
    HTML_("a:my link","<a>my link</a>")
    HTML_("a [href=\"http://...\"] #id :my link","<a href=\"http://...\" id=\"id\">my link</a>")
    //Insignificant whitespace before colon; significant after but first space after is stipped
    HTML_("span :foo","<span>foo</span>");
    HTML_("span : foo","<span>foo</span>");
    HTML_("span :            foo","<span>           foo</span>");
}

BOOST_AUTO_TEST_CASE(header){
    HTML_("# Header 1","<h1>Header 1</h1>")
    HTML_("## Header 2","<h2>Header 2</h2>")
    HTML_("### Header 3","<h3>Header 3</h3>")
    HTML_("#### Header 4","<h4>Header 4</h4>")
    HTML_("##### Header 5","<h5>Header 5</h5>")
    HTML_("###### Header 6","<h6>Header 6</h6>")

    // A single "#" with no space after it is parsed as an `id` directive
    HTML_("#identifier","<div id=\"identifier\" />")

    // Multiple hases with no space after them are parsed as
    // plain text
    HTML_("##plain old text","##plain old text")
    HTML_("####plain old text","####plain old text")
}

BOOST_AUTO_TEST_CASE(ul){
    HTML_("* item 1","<ul>\n\t<li>item 1</li>\n</ul>")
    HTML_("- item 1","<ul>\n\t<li>item 1</li>\n</ul>")
    HTML_("+ item 1","<ul>\n\t<li>item 1</li>\n</ul>")

    HTML_("* item 1\n* item 2","<ul>\n\t<li>item 1</li>\n\t<li>item 2</li>\n</ul>")
}

BOOST_AUTO_TEST_CASE(ol){
    HTML_("1. item 1","<ol>\n\t<li>item 1</li>\n</ol>")
    HTML_("1. item 1\n2. item 2","<ol>\n\t<li>item 1</li>\n\t<li>item 2</li>\n</ol>")

    // Digits do not need to be in order
    HTML_("1. item 1\n1. item 2","<ol>\n\t<li>item 1</li>\n\t<li>item 2</li>\n</ol>")
    HTML_("100. item 1\n1. item 2","<ol>\n\t<li>item 1</li>\n\t<li>item 2</li>\n</ol>")
}

BOOST_AUTO_TEST_CASE(id_class){
    // Shorthand CSS id and class works
    ECHO_("ul #id")
    ECHO_("ul .class")
    // Only one id
    BACK_("ul #id1 #id2","ul #id2")
    // More than one class
    HTML_("div .klass","<div class=\"klass\" />");
    HTML_("div .klass1 .klass2","<div class=\"klass1 klass2\" />");
    HTML_("div .klass-a .klass-b .klass-c","<div class=\"klass-a klass-b klass-c\" />");
    // No need to include div
    ECHO_("#id")
    HTML_(".class","<div class=\"class\" />")
    CILA_("<div class=\"class\" />",".class")
    ECHO_(".class")
    // Mix them up
    ECHO_("#id .class")
    // ... id always comes before class
    BACK_(".class #id","#id .class")
    // Multiple classes
    HTML_(".a .b .c #id","<div class=\"a b c\" id=\"id\" />")
    ECHO_(".a .b .c .d")
}

BOOST_AUTO_TEST_CASE(attributes){
    HTML_("a [href=\"http://stenci.la\"]:Stencila","<a href=\"http://stenci.la\">Stencila</a>");
    ECHO_("a [href=\"http://stenci.la\"] [title=\"Stencila\"]\n\tStencila");
    // More than one
    HTML_("div [attr1=\"1\"] [attr2=\"2\"]","<div attr1=\"1\" attr2=\"2\" />");
    ECHO_("ul [attr1=\"1\"] [attr2=\"2\"] [attr3=\"3\"]");
    // Single quotes are replaced by doubles
    BACK_("span [attr1='value']","span [attr1=\"value\"]")
    // No need to include div
    HTML_("[attr=\"1\"]","<div attr=\"1\" />")
    ECHO_("[attr=\"1\"]");
}

BOOST_AUTO_TEST_CASE(flags){
    HTML_("&tH4dFg","<div data-hash=\"tH4dFg\" />")
    ECHO_("&tH4dFg")

    HTML_("off","<div data-off=\"true\" />")
    ECHO_("off")

    HTML_("@42","<div data-index=\"42\" />")
    ECHO_("@42")

    HTML_("lock","<div data-lock=\"true\" />")
    ECHO_("lock")

    HTML_("out","<div data-out=\"true\" />")
    ECHO_("out")

    ECHO_("&tH4dFg off @42 lock out");
    ECHO_("p &tH4dFg off @42 lock out");
    ECHO_("#id .class &tH4dFg off @42 lock out");
}

BOOST_AUTO_TEST_CASE(paragraph_implied){
    // Paragraph (empty line before)
    HTML_("div\n\nThis should be a paragraph","<div />\n<p>This should be a paragraph</p>")
    // Text (no empty line before)
    HTML_("This should NOT be a paragraph","This should NOT be a paragraph")
    HTML_("div\nThis should NOT be a paragraph","<div />\nThis should NOT be a paragraph")
    // Nests text (no empty line before)
    HTML_("div\n\tThis should NOT be a paragraph","<div>This should NOT be a paragraph</div>")
    // Nested paragraph
    HTML_("div\n\n\tThis should be a paragraph","<div>\n\t<p>This should be a paragraph</p>\n</div>")
}

BOOST_AUTO_TEST_CASE(equations){
    // AsciiMath : lines starting with a | are made into separate paragraphs
    HTML_("|E=mc^2|","<script type=\"math/asciimath\">E=mc^2</script>")
    ECHO_("|E=mc^2|")
    // Tex and LaTeX : lines starting with a \( are made into separate paragraphs
    HTML_("\\(E=mc^2\\)","<script type=\"math/tex\">E=mc^2</script>")
    ECHO_("\\(E=mc^2\\)")
    //...at present inline math should not be parsed, only lines starting with delimiter
    HTML_("p:where |c| is the speed of light","<p>\n\twhere \n\t<script type=\"math/asciimath\">c</script>\n\t is the speed of light\n</p>")
    HTML_("p:where \\(c\\) is the speed of light","<p>where \\(c\\) is the speed of light</p>")
}

BOOST_AUTO_TEST_CASE(meta){
    HTML__("#title: My title","<div id=\"title\">My title</div>")
    HTML__("#description: A short little stencil","<div id=\"description\">A short little stencil</div>")
    HTML__("#keywords: foo,bar","<div id=\"keywords\">foo,bar</div>")
    HTML__(".author: Joe Bloggs","<div class=\"author\">Joe Bloggs</div>")
    HTML__("#contexts: r","<div id=\"contexts\">r</div>")
    HTML__("#theme: beautiful","<div id=\"theme\">beautiful</div>")
}

BOOST_AUTO_TEST_CASE(directive_code_1){
    auto cila_ = 
R"(r
	pi <- 3.14)";
    auto html_ = 
R"(<pre data-code="r">
pi &lt;- 3.14
</pre>)";
    HTML_(cila_,html_)
    ECHO_(cila_)
}

BOOST_AUTO_TEST_CASE(directive_code_2){
    auto cila_ = 
R"(r
	pi <- 3.14
	print(pi))";
    auto html_ = 
R"(<pre data-code="r">
pi &lt;- 3.14
print(pi)
</pre>)";
    HTML_(cila_,html_)
    ECHO_(cila_)
}

BOOST_AUTO_TEST_CASE(directive_code_3){
    auto cila_ = 
R"(r
	pi <- 3.14
	2*pi
	2*pi*r^2
div
div)";
    auto html_ = 
R"(<pre data-code="r">
pi &lt;- 3.14
2*pi
2*pi*r^2
</pre>
<div />
<div />)";
    HTML_(cila_,html_)
    ECHO_(cila_)
}

BOOST_AUTO_TEST_CASE(directive_code_with_empty_lines){
	auto cila_ = 
R"(r
	pi <- 3.14

	2*pi

	2*pi*r^2)";
	auto html_ = 
R"(<pre data-code="r">
pi &lt;- 3.14

2*pi

2*pi*r^2
</pre>)";
	HTML_(cila_,html_)
}

BOOST_AUTO_TEST_CASE(directive_code_with_empty_line_before_next){
	auto cila_ = 
R"(r
	pi <- 3.14
	
div)";
	auto html_ = 
R"(<pre data-code="r">
pi &lt;- 3.14

</pre>
<div />)";
	HTML_(cila_,html_)
}

BOOST_AUTO_TEST_CASE(directive_code_image){
    auto cila_ = 
R"(r png 60x42
	plot(x,y))";
    auto html_ = 
R"(<pre data-code="r" data-format="png" data-size="60x42">
plot(x,y)
</pre>)";
    HTML_(cila_,html_)
    ECHO_(cila_)
}

BOOST_AUTO_TEST_CASE(directive_text){
    HTML_("text variable","<div data-text=\"variable\" />");
    HTML_("span text variable","<span data-text=\"variable\" />");
}

BOOST_AUTO_TEST_CASE(directive_with){
    HTML_("with what","<div data-with=\"what\" />")
    CILA_("<div data-with=\"what\" />","with what")
    ECHO_("with what")

    ECHO_("section!with what")
}

BOOST_AUTO_TEST_CASE(directive_if){
    HTML_(
        "if true\n\tp .a\nelif false\n\tp .b\nelse\n\tp .c",
        "<div data-if=\"true\">\n\t<p class=\"a\" />\n</div>\n<div data-elif=\"false\">\n\t<p class=\"b\" />\n</div>\n<div data-else=\"\">\n\t<p class=\"c\" />\n</div>"
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
R"(<div data-switch="a">
	<div data-case="3.14">Pi</div>
	<div data-case="42">The answer</div>
	<div data-default="">A number</div>
</div>)";
	HTML_(cila_,html_)
    ECHO_(cila_)
}

BOOST_AUTO_TEST_CASE(directive_for){
    ECHO_("for item in items")
    HTML_("for item in items","<div data-for=\"item:items\" />")

    ECHO_("for item in items\n\tp")
}

BOOST_AUTO_TEST_CASE(directive_include){
	ECHO_("include address")
	HTML_("include address","<div data-include=\"address\" />")

	ECHO_("include address selector")

	ECHO_("include a-superbly-sublime-stencil #a-marvelous-macro")
	ECHO_("include a-stencil-with-no-macro-defined .class-a [attr=\"x\"] .class-b")

    // Special '.' identifier for current stencil
    ECHO_("macro hello\n\ttext who\ninclude . #hello\n\tset who = 'world'")

    // Set directive
    ECHO_("include stencil selector\n\tset a = 4\n\tset b = 1")
    ECHO_("include stencil selector\n\tset a = 7\n\tp included\n\t\tSome included text")
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
        ECHO_(modifier+" selector")
        HTML_(modifier+" selector","<div data-"+modifier+"=\"selector\" />")
    }
}

BOOST_AUTO_TEST_CASE(directive_macro){
	ECHO_("macro name")
    HTML_("macro name: {text a+b}","<div data-macro=\"name\" id=\"name\">\n\t<div data-text=\"a+b\" />\n</div>")

    HTML_("macro name\n\tpar x","<div data-macro=\"name\" id=\"name\">\n\t<div data-par=\"x\" />\n</div>")
}

BOOST_AUTO_TEST_CASE(directive_param){
    HTML_("par x","<div data-par=\"x\" />")
    HTML_("par x:text","<div data-par=\"x:text\" />")
    HTML_("par x:number = 42","<div data-par=\"x:number=42\" />")
    HTML_("par x = 'a'","<div data-par=\"x='a'\" />")

    ECHO_("par x")
    ECHO_("par x = 1")
    ECHO_("par x:number = 42")
}

BOOST_AUTO_TEST_CASE(inlines){
    HTML_("Text with a no inlines","Text with a no inlines");
    HTML_("Text with a {a [href=\"http://stencil.la\"]:link} in it.","Text with a \n<a href=\"http://stencil.la\">link</a>\n in it.");

    HTML_("{div}","<div />");
    HTML_("{div:{div}}","<div>\n\t<div />\n</div>");

    HTML_(
        "The minimum is {if a<b :{text a}}{else :{text b}}",
        "The minimum is \n<div data-if=\"a&lt;b\">\n\t<div data-text=\"a\" />\n</div>\n<div data-else=\"\">\n\t<div data-text=\"b\" />\n</div>"
    );

    HTML_("div\n\tSome inline {text pi*2}","<div>\n\tSome inline \n\t<div data-text=\"pi*2\" />\n</div>");

    HTML_("div: Some text","<div>Some text</div>");
    HTML_("div: {Some text}","<div>Some text</div>");
    HTML_("div: Text with a {span: inside span}.","<div>\n\tText with a \n\t<span>inside span</span>\n\t.\n</div>");
}

#undef HTML_
#undef CILA_
#undef ECHO_
#undef BACK_

BOOST_AUTO_TEST_SUITE_END()

