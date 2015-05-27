#include <stencila/version.hpp>
#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

std::string Stencil::html(bool document,bool indent) const {
	if(not document){
		// Return content only
		return Xml::Document::dump(indent);
	} else {
		// Return a complete HTML document
		Html::Document doc;
		typedef Xml::Node Node;

		// Being a valid HTML5 document, doc already has a <head> <title> and <body>
		Node head = doc.find("head");
		Node body = doc.find("body");

		// Properties put into <meta> as microdata
		// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta#attr-itemprop 
		// These are used by `Stencila.launch()` Javascript function to display the
		// component
		head.append("meta",{
			{"itemprop","type"},
			{"content","stencil"}
		});
		head.append("meta",{
			{"itemprop","address"},
			{"content",address()}
		});
		head.append("meta",{
			{"itemprop","theme"},
			{"content",theme()}
		});
		head.append("meta",{
			{"itemprop","closed"},
			{"content",closed()?"true":"false"}
		});
		head.append("meta",{
			{"itemprop","contexts"},
			{"content",join(contexts(),",")}
		});

		// Title is repeated in <title>
		// Although we are creating an XHTML5 document, an empty title tag (i.e <title />)
		// can cause browser parsing errors. So always ensure that there is some title content.
		auto t = title();
		if(t.length()==0) t = "Untitled";
		head.find("title").text(t);

		// Description is repeated in <meta>
		auto d = description();
		if(d.length()>0){
			head.append("meta",{
				{"name","description"},
				{"content",d}
			});
		}

		// Keywords are repeated in <meta>
		auto k = keywords();
		if(k.size()>0){
			head.append("meta",{
				{"name","keywords"},
				{"content",join(k,",")}
			});
		}

		// The following tags are appended with a space as content so that they do not
		// get rendered as empty tags (e.g. <script... />). Whilst empty tags should be OK with XHTML
		// they can cause problems with some browsers.

		/**
		 * <link rel="stylesheet" ...
		 *
		 * Links to CSS stylesheets are [placed in the head](http://developer.yahoo.com/performance/rules.html#css_top) 
		 * Use a site-root relative path (by adding the leading forward slash) so that CSS can be served from 
		 * localhost.
		 */
		std::string css = "/" + theme() + "/theme.min.css";
		head.append("link",{
			{"rel","stylesheet"},
			{"type","text/css"},
			{"href",css}
		}," ");

		/**
		 * Authors are repeated as `<a rel="author" ...>` elements within an `<address>` element.
		 * The placement of `<address>` as a child of `<body>` should mean that this authors list applies to the whole document.
		 * See:
		 *   http://html5doctor.com/the-address-element/
		 *   http://www.w3.org/TR/html5/sections.html#the-address-element
		 *   http://stackoverflow.com/questions/7290504/which-html5-tag-should-i-use-to-mark-up-an-authors-name
		 *   http://stackoverflow.com/a/7295013
		 */
		auto a = authors();
		if(a.size()>0){
			auto authors_elem = body.append("address",{
				{"id","authors"}
			}," ");
			for(auto author : authors()){
				authors_elem.append("a",{
					{"rel","author"},
					{"href","#"}
				},author);
			}
		}        

		/**
		 * #content
		 *
		 * Content is placed in a <main> rather than just using the <body> so that 
		 * extra HTML elements can be added by the theme without affecting the stencil's content.
		 */
		auto content = body.append("main",{
			{"id","content"}
		}," ");
		content.append(*this);

		// Load versioned, minified `stencila.js` from get.stenci.la. This has
		// a "far future" cache header so it should be available even when offline
		body.append("script",{{"src","/build/js/requires.min.js"}}," ");
		body.append("script",{{"src","/build/js/stencila.js"}}," ");
		//std::string js = "//get.stenci.la/js/stencila-"+Stencila::version+".min.js";
		//body.append("script",{
		//	{"src",js}
		//}," ");
		
		// Now Stencila Javascript is loaded, launch the component
		body.append("script","Stencila.launch();");

		// Validate the HTML5 document before dumping it
		doc.validate();
		return doc.dump();
	}
}


Stencil& Stencil::html(const std::string& html){
	// Clear content before appending new content from Html::Document
	clear();
	Html::Document doc(html);
	auto body = doc.find("body");
	if(auto elem = body.find("main","id","content")){
		append_children(elem);
	}
	else append_children(doc.find("body"));
	return *this;
}

} //namespace Stencila
