#include <stencila/version.hpp>
#include <stencila/component.hpp>
#include <stencila/html.hpp>

namespace Stencila {

template<class Type>
Html::Document Component_page_doc(const Type& component) {
	using namespace Html;

	// Return a complete HTML document
	// Being a valid HTML5 document, doc already has a <head> <title> and <body>
	Document doc;
	Node head = doc.find("head");
	Node body = doc.find("body");

	// For potential use in resolving Stencila version differences
	// include a <meta> generator tag
	head.append("meta",{
		{"name","generator"},
		{"content","Stencila "+version}
	});

	// For layout that is responsive to the device size
	// include a <meta> viewport tag
	head.append("meta",{
		{"name","viewport"},
		{"content","width=device-width, initial-scale=1"}
	});

	// Component properties put into <meta> as microdata
	// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta#attr-itemprop 
	// These are used by `Stencila.launch()` Javascript function to display the
	// component
	auto type = lower(Component::type_to_string(component.type()));
	head.append("meta",{
		{"itemprop","type"},
		{"content",type}
	});
	head.append("meta",{
		{"itemprop","address"},
		{"content",component.address()}
	});
	head.append("meta",{
		{"itemprop","version"},
		{"content",component.version()}
	});

	// Title is put in <title>
	// Although we are creating an XHTML5 document, an empty title tag (i.e <title />)
	// can cause browser parsing errors. So always ensure that there is some title content.
	auto title = component.title();
	if(title.length()==0) title = "Untitled";
	head.find("title").text(title);

	// Description is put in <meta>
	auto description = component.description();
	if(description.length()>0){
		head.append("meta",{
			{"name","description"},
			{"content",description}
		});
	}

	// Keywords are put in <meta>
	auto keywords = component.keywords();
	if(keywords.size()>0){
		head.append("meta",{
			{"name","keywords"},
			{"content",join(keywords,",")}
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

	// A fallback function to load the theme CSS from http://stenci.la if it is not served from the 
	// host of this HTML (e.g. file:// or some non-Stencila-aware server)
	// To prevent flash of unstyled content (FOUC) while the new <link> is loading make the document class 'unready'
	// and then remove this class when the style is loaded (there is a fallback to this fallback at end of document).
	// See http://www.techrepublic.com/blog/web-designer/how-to-prevent-flash-of-unstyled-content-on-your-websites/
	std::string fallback = "function css_fallback(c){";
	fallback += "var d=document,l;";
	fallback += "l=d.createElement('link');l.rel='stylesheet';l.type='text/css';l.href=c;";
	fallback += "d.documentElement.className='unready';l.onload=function(){d.documentElement.className='';};";
	fallback += "d.getElementsByTagName('head')[0].appendChild(l);";
	fallback += "};";
	// Add CSS fallback Javascript
	head.append("script",{{"type","application/javascript"}},fallback);
	// Add CSS fallback style for the unready document
	head.append("style",{{"type","text/css"}},".unready{display:none;}");

	head.append("link",{
		{"rel", "stylesheet"},
		{"type", "text/css"},
		{"href", "/get/web/" + type + ".min.css"},
		{"onerror", "css_fallback('https://stenci.la/get/web/" + type + ".min.css')"}
	}," ");

	/**
	 * Authors are inserted as `<a rel="author" ...>` elements within an `<address>` element.
	 * The placement of `<address>` as a child of `<body>` should mean that this authors list applies to the whole document.
	 * See:
	 *   http://html5doctor.com/the-address-element/
	 *   http://www.w3.org/TR/html5/sections.html#the-address-element
	 *   http://stackoverflow.com/questions/7290504/which-html5-tag-should-i-use-to-mark-up-an-authors-name
	 *   http://stackoverflow.com/a/7295013
	 */
	auto authors = component.authors();
	if(authors.size()>0){
		auto authors_elem = body.append("address",{
			{"id","authors"}
		}," ");
		for(auto author : authors){
			authors_elem.append("a",{
				{"rel","author"},
				{"href","#"}
			},author);
		}
	}

	// Main element where custom component pages should add content
	body.append("main",{{"id","main"}});

	// Load Stencila Javascript
	std::string js = "/get/web/"+type+".min.js";
	// First attempt to load from host
	body.append("script",{{"src",js}}," ");
	// Fallback load from https://stenci.la. This is https:// not a "propocol relative URL" so that it 
	// will work with file:// and https:// (i.e not mixed content as it would be if it were http://)
	body.append(
		"script",
		{{"type","application/javascript"}},
		std::string("if(!window.Stencila){") +
			"window.StencilaHost=\"https://stenci.la\";" + 
			"document.write(unescape('%3Cscript src=\"https://stenci.la"+js+"\"%3E%3C/script%3E'))" + 
		"}"
	);
	
	// Fallback to the CSS fallback! Remove the `unready` class from the root element is not already
	// removed. This is in case the remote CSS link added by the CSS fallback function (see above) fails to load.
	body.append("script",std::string("window.setTimeout(function(){")+
		"document.documentElement.className='';" + 
		"if(!window.Stencila){window.alert('Page could not be fully loaded. Not all functionality will be available.');}"+
	"},10000)");

	return doc;
}

}