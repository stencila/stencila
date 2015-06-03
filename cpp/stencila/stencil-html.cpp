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
		 */
		std::string css = "/" + theme() + "/theme.min.css";
		head.append("link",{
			{"rel","stylesheet"},
			{"type","text/css"},
			{"href",css}
		}," ");

		// A fallback function to load the theme CSS from http://stenci.la if it is not served from the 
		// host of this HTML (e.g. file:// or some non-Stencila-aware server)
		std::string fallback = "(function(c){";
		// Local variables
		fallback += "var d=document,s,i,l;";
		// Check to see if the theme stylesheet has been loaded
		// The theme CSS will not necessarily be the first stylesheet and no all stylesheets have `href`
		// The try block is to avoid the security error raised by Firefox for accessing cross domain stylesheets
		// If this happens it means the theme CSS was not loaded from the current domain so do not return
		// See http://stackoverflow.com/questions/21642277/security-error-the-operation-is-insecure-in-firefox-document-stylesheets
		// Note use of `!=` instead of `<` to avoid escaping in generated HTML
		fallback += "s=d.styleSheets;for(i=0;i!=s.length;i++){if((s[i].href||'').match(c)){try{if(s[i].cssRules.length)return;}catch(e){}}}";
		// If still in the function the stylesheet must not have been loaded so create
		// a new <link> to the theme CSS on http://stenci.la
		fallback += "l=d.createElement('link');l.rel='stylesheet';l.type='text/css';l.href='https://stenci.la'+c;";
		// To prevent flash of unstyled content (FOUC) while the new <link> is loading make the document class 'unready'
		// and then remove this class when the style is loaded (there is a fallback to this fallback at end of document).
		// See http://www.techrepublic.com/blog/web-designer/how-to-prevent-flash-of-unstyled-content-on-your-websites/
		fallback += "d.documentElement.className='unready';l.onload=function(){d.documentElement.className='';};";
		// Append new link to head
		fallback += "d.getElementsByTagName('head')[0].appendChild(l);";
		// Call the function
		fallback += "})('"+css+"');";
		// Add CSS fallback Javascript
		head.append("script",{{"type","text/javascript"}},fallback);
		// Add CSS fallback style for the unready document
		head.append("style",{{"type","text/css"}},".unready{display:none;}");

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

		// Load Stencila Javascript module
		// Use version number to check if in development. No development versions
		// of stencila.js are on get.stenci.la (only release versions).
		bool development = Stencila::version.find("-")!=std::string::npos;
		if(development){
			// Load development version from the current host (usually http://localhost:7373)
			// Requires that the `make build-serve ...` task has been run so that build directory
			// of the `stencila/stencila` repo is being served and that `make js-develop` task has been 
			// run to ensure the following files are in that directory
			body.append("script",{{"src","/build/js/requires.min.js"}}," ");
			body.append("script",{{"src","/build/js/stencila.js"}}," ");
		} else {
			// Load versioned, minified file from get.stenci.la. This has
			// a "far future" cache header so it should be available even when offline
			body.append("script",{{"src","//get.stenci.la/js/stencila-"+Stencila::version+".min.js"}}," ");
		}		
		
		// Launch the component
		body.append("script","Stencila.launch();");

		// Fallback to the CSS fallback! Remove the `unready` class from the root element is not already
		// removed. This is in case the remote CSS link added by the CSS fallback function (see above) fails to load.
		// This could perhaps generate an alert
		body.append("script",std::string("window.setTimeout(function(){")+
			"document.documentElement.className='';" + 
			"if(!window.Stencila){window.alert('Page could not be fully loaded. Not all functionality will be available.');}"+
		"},10000)");

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
