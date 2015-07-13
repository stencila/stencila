#include <iostream>

#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <stencila/html.hpp>

BOOST_AUTO_TEST_SUITE(html_quick)

using namespace Stencila::Html;

BOOST_AUTO_TEST_CASE(tidy){
	// Tests mainly for understanding/checkingwhat htmltidy does
	boost::regex regex("<body>(.*)</body>");
	boost::smatch match;
	#define CHECK(in,out) { \
		auto temp = Fragment::tidy(in); \
		boost::regex_search(temp,match,regex); \
		BOOST_CHECK_EQUAL(match[1].str(),out); \
	}

	CHECK(
		"<p>Cheking works</p>",
		"<p>Cheking works</p>"
	)

	// htmltidy 5.0.0RC1 (and before) puts start and end newlines in pre and script elements
	// See See https://github.com/htacg/tidy-html5/issues/158 and https://github.com/htacg/tidy-html5/issues/227
	// Our pull request https://github.com/htacg/tidy-html5/pull/228 removes them if `vertical-space` is no
	// But any intentional initial newline is lost (in htmltidy's parsing?)
	CHECK(
		"<pre>code</pre>",
		"<pre>code</pre>"
	)
	CHECK(
		"<pre>\ncode</pre>",
		"<pre>code</pre>"
	)
	CHECK(
		"<pre>\n\ncode</pre>",
		"<pre>\ncode</pre>"
	)


	// htmltidy does not allow top level scripts, they must be within something
	CHECK(
		"<script>code</script>",
		""
	)
	CHECK(
		"<div><script>code</script></div>",
		"<div><script>code</script></div>"
	)

	#undef CHECK 
}
 
BOOST_AUTO_TEST_CASE(load_and_dump){
	// Tests of both parsing/tidying from HTML string and dumping to HTML string
	#define CHECK(in,out) BOOST_CHECK_EQUAL(Fragment().load(in).dump(),out);

	// Recognises <main> 
	CHECK(
		"<main id=\"content\">\n\tcontent\n</main>",
		"<main id=\"content\">\n\tcontent\n</main>"
	)

	// Fixes nmatched tags
	CHECK(
		"<h2>subheading</h3>",
		"<h2>subheading</h2>"
	)

	// Fixes missing end tags
	CHECK(
		"<p class=\"message\">Don't panic!",
		"<p class=\"message\">\n\tDon't panic!\n</p>"
	)

	// Preserves tabs in <pre> elements
	CHECK(
		"<pre>\tline1\n\t\tline2\n</pre>",
		"<pre>\tline1\n\t\tline2\n</pre>"
	)
	CHECK(
		"<pre id=\"id\">\tline1\n\t\tline2\n</pre>",
		"<pre id=\"id\">\tline1\n\t\tline2\n</pre>"
	)

	// Doesn't add CDATA wrapper to script elements
	CHECK(
		"<div><script>code</script></div>",
		"<div>\n\t<script>code</script>\n</div>"
	)

	// Does not have any newline in inline math elements but does in display mode ones
	CHECK(
		"<div><script type=\"math/asciimath\">\nE=mc^2\n</script></div>",
		"<div>\n\t<script type=\"math/asciimath\">E=mc^2</script>\n</div>"
	)
	CHECK(
		"<div><script type=\"math/asciimath; mode=display\">\n\nE=mc^2\n</script></div>",
		"<div>\n\t<script type=\"math/asciimath; mode=display\">\nE=mc^2\n</script>\n</div>"
	)

	#undef CHECK 
}

BOOST_AUTO_TEST_CASE(make_and_dump){
	// Tests of creating using XML::Node methods and then dumping to HTML string
	Fragment frag;

	// Escapes properly
	frag.clear();
	frag.append("span",{{"data-write","\"a quoted value\""}},"a < b & c < d");
	BOOST_CHECK_EQUAL(
		frag.dump(),
		R"(<span data-write="&quot;a quoted value&quot;">a &lt; b &amp; c &lt; d</span>)"
	);

	// Outputs scripts
	frag.clear();
	frag.append("script",{{"type","text/javascript"}},"code");
	BOOST_CHECK_EQUAL(
		frag.dump(),
		R"(<script type="text/javascript">code</script>)"
	);
}

/**
 * Test escaping of text in attributes and nodes
 *
 * Without proper escaping a user could insert text that you be used in a 
 * XSS attack
 */
BOOST_AUTO_TEST_CASE(escaping){
	Document doc;

	// Element text
	BOOST_CHECK_EQUAL(
		doc.append("div","<script>alert('xss')</script>").dump(),
		"<div>&lt;script&gt;alert('xss')&lt;/script&gt;</div>"
	);

	// Element attributes
	BOOST_CHECK_EQUAL(
		doc.append("div",{{"class","foo\" onmouseover=\"alert('xss')"}}).dump(),
		"<div class=\"foo&quot; onmouseover=&quot;alert('xss')\" />"
	);
}

/**
 * Test common Cross Site Scripting (XSS) attack vectors
 * 
 * These tests simply "quantify" how our HTML implementation (ie. tidy-html5) parses
 * the types of HTML fragments commonly used in XSS attacks.
 * 
 * Most of these examples are taken from https://www.owasp.org/index.php/XSS_Filter_Evasion_Cheat_Sheet
 * The focus has been on implementing checks for XSS attacks exploit quirks in the parsing of malformed HTML.
 * 
 * The Html::sanitize() method deals with actually attempting to remove the attack vectors (using whitelists)
 */
BOOST_AUTO_TEST_CASE(xss){

	//tidy-html5 ignores some elements (e.g. <script>) at top level; so wrap them in a <body>
	#define CHECK(in,out) BOOST_CHECK_EQUAL(Fragment(in).dump(false),out);

	// XSS Locator
	BOOST_CHECK_THROW(
		Document("'';!--\"<XSS>=&{()}"),
		Stencila::Exception
	)

	// No Filter Evasion
	CHECK(
		"<div><script src=\"http://example.com/xss.js\" /></div>",
		"<div><script src=\"http://example.com/xss.js\"></script></div>" 
	)
	CHECK(
		"<div><script>alert('XSS')</script></div>",
		"<div><script>alert('XSS')</script></div>"
	)

	// Image XSS using the JavaScript directive
	CHECK(
		"<img src=\"javascript:alert('XSS');\">",
		"<img src=\"javascript:alert('XSS');\">"
	)

	// Malformed IMG tags
	CHECK(
		R"( <img """><SCRIPT>alert('XSS')</SCRIPT>"> )",
		"<img><script>alert('XSS')</script>\"&gt;"
	)

	// Default SRC tag by leaving it empty
	CHECK(
		"<img src= onmouseover=\"alert('XSS')\">",
		"<img src=\"onmouseover=&quot;alert('XSS')&quot;\">"
	)

	// Default SRC tag by leaving it out entirely
	CHECK(
		"<img onmouseover=\"alert('XSS')\">",
		"<img onmouseover=\"alert('XSS')\">"
	)

	// Decimal HTML character references
	CHECK(
		"<img src=&#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;&#58;&#97;&#108;&#101;&#114;&#116;&#40;&#39;&#88;&#83;&#83;&#39;&#41;>",
		"<img src=\"javascript:alert('XSS')\">"
	)

	// Decimal HTML character references without trailing semicolons
	CHECK(
		"<img src=&#0000106&#0000097&#0000118&#0000097&#0000115&#0000099&#0000114&#0000105&#0000112&#0000116&#0000058&#0000097&#0000108&#0000101&#0000114&#0000116&#0000040&#0000039&#0000088&#0000083&#0000083&#0000039&#0000041>",
		"<img src=\"javascript:alert('XSS')\">"
	)

	// Hexadecimal HTML character references without trailing semicolons
	CHECK(
		"<img src=&#x6A&#x61&#x76&#x61&#x73&#x63&#x72&#x69&#x70&#x74&#x3A&#x61&#x6C&#x65&#x72&#x74&#x28&#x27&#x58&#x53&#x53&#x27&#x29>",
		"<img src=\"javascript:alert('XSS')\">"
	)

	// Embedded tab
	CHECK(
		"<img src=\"jav\tascript:alert('XSS');\">",
		"<img src=\"jav%20ascript:alert('XSS');\">"
	)

	// Embedded Encoded tab
	CHECK(
		"<img src=\"jav&#x09;ascript:alert('XSS');\">",
		"<img src=\"jav%09ascript:alert('XSS');\">"
	)

	// Embedded newline to break up XSS
	CHECK(
		"<img src=\"jav&#x0A;ascript:alert('XSS');\">",
		"<img src=\"jav%20ascript:alert('XSS');\">"
	)

	// Embedded carriage return to break up XSS
	CHECK(
		"<img src=\"jav&#x0D;ascript:alert('XSS');\">",
		"<img src=\"jav%0Dascript:alert('XSS');\">"
	)

	// Spaces and meta chars before the JavaScript in images for XSS
	CHECK(
		"<img src=\" &#14;  javascript:alert('XSS');\">",
		"<img src=\"%0E%20javascript:alert('XSS');\">"
	)

	// Non-alpha-non-digit XSS
	CHECK(
		"<div><script/XSS src=\"http://ha.ckers.org/xss.js\"></script></div>",
		"<div><script src=\"http://ha.ckers.org/xss.js\"></script></div>"
	)
	CHECK(
		"<img onmouseover!#$%&()*~+-_.,:;?@[/|\\]^`=alert(\"XSS\")>",
		"<img>"
	)
	CHECK(
		"<div><script/src=\"http://ha.ckers.org/xss.js\"></script></div>",
		"<div><script></script></div>"
	)

	// Extraneous open brackets
	CHECK(
		"<div><<script>alert(\"XSS\");//<</script></div>",
		"<div>&lt;&lt;script&gt;alert(\"XSS\");//&lt;&lt;/script&gt;</div>"
	)    

	// No closing script tags
	CHECK(
		"<div><script src=http://ha.ckers.org/xss.js?< B ></div>",
		"<div><script src=\"http://ha.ckers.org/xss.js?\">&lt; B &gt;&lt;/div&gt;</script></div>"
	)        

	// Protocol resolution in script tags
	CHECK(
		"<div><script src=//ha.ckers.org/.j></div>",
		"<div><script src=\"//ha.ckers.org/.j\"></script></div>"
	)

	// Half open HTML/JavaScript XSS vector
	CHECK(
		"<img src=\"javascript:alert('XSS')\"",
		"<img src=\"javascript:alert('XSS')\">"
	)

	// Double open angle brackets
	CHECK(
		"<iframe src=http://ha.ckers.org/scriptlet.html <",
		"<iframe src=\"http://ha.ckers.org/scriptlet.html\"></iframe>"
	)

	// STYLE attribute using a comment to break up expression
	CHECK(
		"<img style=\"xss:expr/*XSS*/ession(alert('XSS'))\">",
		"<img style=\"xss:expr/*XSS*/ession(alert('XSS'))\">"
	)   

	#undef CHECK

	// The following need to be done on a doc head (for each, dumped as XML)
	#define CHECK(in,out) BOOST_CHECK_EQUAL(Document(in).select("head meta").dump(),out);

	// META using data
	CHECK(
		"<META HTTP-EQUIV=\"refresh\" CONTENT=\"0;url=data:text/html base64,PHNjcmlwdD5hbGVydCgnWFNTJyk8L3NjcmlwdD4K\">",
		"<meta http-equiv=\"refresh\" content=\"0;url=data:text/html base64,PHNjcmlwdD5hbGVydCgnWFNTJyk8L3NjcmlwdD4K\" />"
	)  

	// META with additional URL parameter
	CHECK(
		"<META HTTP-EQUIV=\"refresh\" CONTENT=\"0; URL=http://;URL=javascript:alert('XSS');\">",
		"<meta http-equiv=\"refresh\" content=\"0; URL=http://;URL=javascript:alert('XSS');\" />"
	)  

	#undef CHECK
}


BOOST_AUTO_TEST_CASE(doc_not_pretty){
	Document doc;
	BOOST_CHECK_EQUAL(
		doc.dump(false),
		R"(<!DOCTYPE html><html><head><title></title><meta charset="utf-8"></head><body></body></html>)"
	);
}

BOOST_AUTO_TEST_CASE(doc_pretty){
	auto html = R"(<!DOCTYPE html>
<html>
	<head>
		<title>Title</title>
		<meta charset="utf-8">
	</head>
	<body>
		<div>
			<ul>
				<li>One</li>
				<li>Two</li>
				<li>Three</li>
			</ul>
		</div>
	</body>
</html>)";

	Document doc(html);
	BOOST_CHECK_EQUAL(
		doc.dump(),
		html
	);
}

BOOST_AUTO_TEST_CASE(doc_write_read){
	auto tempfile = "/tmp/"+boost::filesystem::unique_path().string();

	Document doc1;
	doc1.find("body").append("p",{{"class","message"}},"Don't panic!");
	doc1.write(tempfile);

	Document doc2;
	doc2.read(tempfile);

	BOOST_CHECK_EQUAL(doc1.dump(),doc2.dump());

	boost::filesystem::remove(tempfile);
}

BOOST_AUTO_TEST_CASE(doc_1){
	Document doc;
	doc.read("html-doc-1.html");
	doc.write("html-doc-1-got.html");

	std::ifstream file("html-doc-1.html");
	std::stringstream html;
	html<<file.rdbuf();

	BOOST_CHECK_EQUAL(doc.dump(),html.str());
}


BOOST_AUTO_TEST_SUITE_END()
 