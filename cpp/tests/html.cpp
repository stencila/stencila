#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/html.hpp>

BOOST_AUTO_TEST_SUITE(html_quick)

using namespace Stencila::Html;
 
BOOST_AUTO_TEST_CASE(load){

	#define CHECK(in,out) BOOST_CHECK_EQUAL(Document(in).find("body").dump_children(),out);

	CHECK(
		"<h2>subheading</h3>",
		"<h2>subheading</h2>"
	)

	CHECK(
		"<main id=\"content\">content</main>",
		"<main id=\"content\">content</main>"
	)

	CHECK(
		"<p class=\"message\">Don't panic!",
		"<p class=\"message\">Don't panic!</p>"
	) 

	// Check for preservation of tabs in <pre> elements
	CHECK(
		"<pre>\n\tline1\n\t\tline2\n</pre>",
		"<pre>\n\tline1\n\t\tline2\n</pre>"
	)
	CHECK(
		"<pre id=\"id\">\n\tline1\n\t\tline2\n</pre>",
		"<pre id=\"id\">\n\tline1\n\t\tline2\n</pre>"
	)

	#undef CHECK 
}

BOOST_AUTO_TEST_CASE(dump_not_pretty){
	Document doc;
	BOOST_CHECK_EQUAL(doc.dump(false),R"(<!DOCTYPE html><html xmlns="http://www.w3.org/1999/xhtml"><head><title></title><meta charset="utf-8"></head><body></body></html>)");
}

BOOST_AUTO_TEST_CASE(dump_pretty){
	Document doc(R"(
		<p>Text <span>in spans</span> and <a href="">in-links</a><span>too</span> should be inline</p>
		<script type="text/asciimath"><![CDATA[e=mc^2]]></script>
		<style type="text/css"><![CDATA[a {color:red;}]]></style>
	)");
	auto result = R"(<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
	<head>
		<title></title>
		<meta charset="utf-8">
	</head>
	<body>
		<p>
			Text <span>in spans</span> and <a href="">in-links</a><span>too</span> should be inline
		</p><script type="text/asciimath">e=mc^2</script>
		<style type="text/css">a {color:red;}</style>
	</body>
</html>)";
	BOOST_CHECK_EQUAL(
		doc.dump(),
		result
	);
	//"
}

BOOST_AUTO_TEST_CASE(dump_escapes){
	Document doc;
	doc.find("body").append("span",{{"data-write","\"a quoted value\""}},"a < b & c < d");
	BOOST_CHECK_EQUAL(
		doc.dump(false),
		R"(<!DOCTYPE html><html xmlns="http://www.w3.org/1999/xhtml"><head><title></title><meta charset="utf-8"></head><body><span data-write="&quot;a quoted value&quot;">a &lt; b &amp; c &lt; d</span></body></html>)"
	);
}


BOOST_AUTO_TEST_CASE(write_read){
	auto tempfile = "/tmp/"+boost::filesystem::unique_path().string();

	Document doc1;
	doc1.find("body").append("p",{{"class","message"}},"Don't panic!");
	doc1.write(tempfile);

	Document doc2;
	doc2.read(tempfile);

	BOOST_CHECK_EQUAL(doc1.dump(),doc2.dump());

	boost::filesystem::remove(tempfile);
}

/**
 * Test that tidy wraps script code in CDATA element
 */
BOOST_AUTO_TEST_CASE(cdata){
	Document doc("<script>code</script>");
	BOOST_CHECK_EQUAL(
		doc.find("script").dump(),
		"<script><![CDATA[\ncode\n]]></script>"
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
	#define CHECK(in,out) BOOST_CHECK_EQUAL(Document(std::string("<body>")+in+"</body>").find("body").dump_children(),out);

	// XSS Locator
	BOOST_CHECK_THROW(
		Document("'';!--\"<XSS>=&{()}"),
		Stencila::Exception
	)

	// No Filter Evasion
	CHECK(
		"<script src=\"http://example.com/xss.js\" />",
		"<script src=\"http://example.com/xss.js\" />" 
	)
	CHECK(
		"<script>alert('XSS')</script>",
		"<script><![CDATA[\nalert('XSS')\n]]></script>"
	)

	// Image XSS using the JavaScript directive
	CHECK(
		"<img src=\"javascript:alert('XSS');\">",
		"<img src=\"javascript:alert('XSS');\" />"
	)

	// Malformed IMG tags
	CHECK(
		R"( <img """><SCRIPT>alert('XSS')</SCRIPT>"> )",
		"<img /><script><![CDATA[\nalert('XSS')\n]]></script>\"&gt;\n"
	)

	// Default SRC tag by leaving it empty
	CHECK(
		"<img src= onmouseover=\"alert('XSS')\">",
		"<img src=\"onmouseover=&quot;alert('XSS')&quot;\" />"
	)

	// Default SRC tag by leaving it out entirely
	CHECK(
		"<img onmouseover=\"alert('XSS')\">",
		"<img onmouseover=\"alert('XSS')\" />"
	)

	// Decimal HTML character references
	CHECK(
		"<img src=&#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;&#58;&#97;&#108;&#101;&#114;&#116;&#40;&#39;&#88;&#83;&#83;&#39;&#41;>",
		"<img src=\"javascript:alert('XSS')\" />"
	)

	// Decimal HTML character references without trailing semicolons
	CHECK(
		"<img src=&#0000106&#0000097&#0000118&#0000097&#0000115&#0000099&#0000114&#0000105&#0000112&#0000116&#0000058&#0000097&#0000108&#0000101&#0000114&#0000116&#0000040&#0000039&#0000088&#0000083&#0000083&#0000039&#0000041>",
		"<img src=\"javascript:alert('XSS')\" />"
	)

	// Hexadecimal HTML character references without trailing semicolons
	CHECK(
		"<img src=&#x6A&#x61&#x76&#x61&#x73&#x63&#x72&#x69&#x70&#x74&#x3A&#x61&#x6C&#x65&#x72&#x74&#x28&#x27&#x58&#x53&#x53&#x27&#x29>",
		"<img src=\"javascript:alert('XSS')\" />"
	)

	// Embedded tab
	CHECK(
		"<IMG SRC=\"jav\tascript:alert('XSS');\">",
		"<img src=\"jav%20ascript:alert('XSS');\" />"
	)

	// Embedded Encoded tab
	CHECK(
		"<IMG SRC=\"jav&#x09;ascript:alert('XSS');\">",
		"<img src=\"jav%09ascript:alert('XSS');\" />"
	)

	// Embedded newline to break up XSS
	CHECK(
		"<IMG SRC=\"jav&#x0A;ascript:alert('XSS');\">",
		"<img src=\"jav%20ascript:alert('XSS');\" />"
	)

	// Embedded carriage return to break up XSS
	CHECK(
		"<IMG SRC=\"jav&#x0D;ascript:alert('XSS');\">",
		"<img src=\"jav%0Dascript:alert('XSS');\" />"
	)

	// Spaces and meta chars before the JavaScript in images for XSS
	CHECK(
		"<IMG SRC=\" &#14;  javascript:alert('XSS');\">",
		"<img src=\"%0E%20javascript:alert('XSS');\" />"
	)

	// Non-alpha-non-digit XSS
	CHECK(
		"<SCRIPT/XSS SRC=\"http://ha.ckers.org/xss.js\"></SCRIPT>",
		"<script src=\"http://ha.ckers.org/xss.js\" />"
	)
	CHECK(
		"<img onmouseover!#$%&()*~+-_.,:;?@[/|\\]^`=alert(\"XSS\")>",
		"<img />"
	)
	CHECK(
		"<SCRIPT/SRC=\"http://ha.ckers.org/xss.js\"></SCRIPT>",
		"<script />"
	)

	// Extraneous open brackets
	CHECK(
		"<<SCRIPT>alert(\"XSS\");//<</SCRIPT>",
		"\n&lt;&lt;SCRIPT&gt;alert(\"XSS\");//&lt;&lt;/SCRIPT&gt;\n"
	)    

	// No closing script tags
	CHECK(
		"<SCRIPT SRC=http://ha.ckers.org/xss.js?< B >",
		"<script src=\"http://ha.ckers.org/xss.js?\"><![CDATA[\n< B ></body>\n]]></script>"
		// Note that when there is no closing script tag the rest of the document is enclosed 
		// in CDATA
	)        

	// Protocol resolution in script tags
	CHECK(
		"<SCRIPT SRC=//ha.ckers.org/.j>",
		"<script src=\"//ha.ckers.org/.j\" />"
	)

	// Half open HTML/JavaScript XSS vector
	CHECK(
		"<IMG SRC=\"javascript:alert('XSS')\"",
		"<img src=\"javascript:alert('XSS')\" />"
	)

	// Double open angle brackets
	CHECK(
		"<iframe src=http://ha.ckers.org/scriptlet.html <",
		"<iframe src=\"http://ha.ckers.org/scriptlet.html\">&lt;&lt;/body&gt;</iframe>"
	)

	// STYLE attribute using a comment to break up expression
	CHECK(
		"<IMG STYLE=\"xss:expr/*XSS*/ession(alert('XSS'))\">",
		"<img style=\"xss:expr/*XSS*/ession(alert('XSS'))\" />"
	)   

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

BOOST_AUTO_TEST_SUITE_END()
 