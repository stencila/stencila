#include <iostream>

#include <boost/filesystem.hpp>

#include <stencila/theme.hpp>
#include <stencila/stencil.hpp>
#include <stencila/helpers.hpp>
#include <stencila/version.hpp>

namespace Stencila {

Theme& Theme::initialise(const std::string& from){
	if(boost::filesystem::exists(from)){
		read(from);
	}
	else {
		std::string path = Component::locate(from);
		if(path.length()) read(path);
		else STENCILA_THROW(Exception,"No theme found with path or address matching from parameter.\n  from: "+from);
	}        
	return *this;
}

std::string Theme::style(void) const {
	return style_;
}

Theme& Theme::style(const std::string& path){
	style_ = path;
	return *this;
}

std::string Theme::behaviour(void) const {
	return behaviour_;
}

Theme& Theme::behaviour(const std::string& path){
	behaviour_ = path;
	return *this;
}

Theme& Theme::read(const std::string& directory){
	Component::read(directory);
	for(std::string file : {"theme.css","theme.scss"}){
		boost::filesystem::path filename = boost::filesystem::path(path()) / file;
		if(boost::filesystem::exists(filename)){
			style(file);
			break;
		}
	}
	for(std::string file : {"theme.js"}){
		boost::filesystem::path filename = boost::filesystem::path(path()) / file;
		if(boost::filesystem::exists(filename)){
			behaviour(file);
			break;
		}
	}
	
	boost::filesystem::path metafile = boost::filesystem::path(path()) / "meta.json";
	if(boost::filesystem::exists(metafile)){
		Json::Document json;
		json.read(metafile.string());
		if(json.has("title")) title_ = json["title"].as<std::string>();
		if(json.has("description")) description_ = json["description"].as<std::string>();
		if(json.has("theme")) theme_ = json["theme"].as<std::string>();
	}

	return *this;
}

Theme& Theme::compile(void) {
	auto home = boost::filesystem::path(path(true));
	if(style_.length()){
		// Convert CSS or SCSS to compressed CSS using SASS
		auto script = Helpers::script("theme-make-mincss.js",R"(
			var sass = require('node-sass');
			var fs = require('fs');
			var args = process.argv.slice(2); // Remove "node" and <script name> args

			var from = args[0];
			var to = args[1];

			var result = sass.renderSync({
			    file: from,
			    // includePaths is an Array of path Strings to look for any @imported files
			    includePaths: ['.'],
			    // outFile specifies where the CSS will be saved. 
			    // This option does not actually output a file, 
			    // but is used as input for generating a source map.
			    outFile: to,
			    // outputStyle is a String to determine how the final 
			    // CSS should be rendered. Its value should be one of
			    // 'nested' or 'compressed'. 
			    outputStyle: 'compressed',
			    // error is a Function to be called upon occurrence of 
			    // an error when rendering the scss to css
			    error: function(error) {
			        console.log(error.message);
			        console.log(error.status);
			        console.log(error.line);
			        console.log(error.column);
			    },
			});
			fs.writeFile(to, result.css);
			console.log(result.stats);
		)");
		Helpers::execute("node '"+script+"' '"+(home/style_).string()+"' '"+(home/"theme.min.css").string()+"'");
	}
	if(behaviour_.length()){
		// Convert JS to compressed JS using UglifyJS
		Helpers::execute("uglifyjs '"+(home/behaviour_).string()+"' -m > '"+(home/"theme.min.js").string()+"'");
	}
	// Generate a preview
	preview((home/"preview.png").string());
	// Generate a static page
	write_to("theme.html",page());
	
	return *this;
}


std::string Theme::serve(void){
	return Component::serve(ThemeType);
}

Theme& Theme::view(void){
	Component::view(ThemeType);
	return *this;
}

Theme& Theme::preview(const std::string& path){
	Component::preview(ThemeType,path);
	return *this;
}

std::string Theme::page(const Component* component){
	return static_cast<const Theme&>(*component).page();
}

std::string Theme::page(void) const {
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
		{"content","theme"}
	});
	head.append("meta",{
		{"itemprop","address"},
		{"content",address()}
	});
	head.append("meta",{
		{"itemprop","theme"},
		{"content",theme()}
	});

	// Title is repeated in <title>
	// Although we are creating an XHTML5 document, an empty title tag (i.e <title />)
	// can cause browser parsing errors. So always ensure that there is some title content.
	auto t = std::string(title());
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
	head.append("link",{
		{"rel","stylesheet"},
		{"type","text/css"},
		{"href",theme()+"theme.min.css"}
	}," ");

	body.append("pre",{
		{"id","style"},
		{"class","code"}
	},read_from(style_));

	body.append("pre",{
		{"id","behaviour"},
		{"class","code"}
	},read_from(behaviour_));

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

	// Validate the HTML5 document before dumping it
	doc.validate();
	return doc.dump();
}

std::string Theme::call(Component* component, const Call& call){
	return static_cast<Theme&>(*component).call(call);
}

std::string Theme::call(const Call& call) {
	auto what = call.what();
	return "";
}

}
