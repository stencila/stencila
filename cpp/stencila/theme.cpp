#include <iostream>

#include <boost/filesystem.hpp>

#include <stencila/component-page.hpp>
#include <stencila/theme.hpp>
#include <stencila/stencil.hpp>
#include <stencila/helpers.hpp>
#include <stencila/version.hpp>

namespace Stencila {

Theme::Theme(void){
}

Theme::Theme(const std::string& from){
	initialise(from);
}

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
	// Get base document
	Html::Document doc = Component_page_doc<Theme>(*this);
	Html::Node head = doc.find("head");
	Html::Node body = doc.find("body");

	// Add style and behaviour before lauch script
	Html::Node main = body.prepend("main");

	main.append("pre",{
		{"id","style"},
		{"class","code"}
	},read_from(style_));

	main.append("pre",{
		{"id","behaviour"},
		{"class","code"}
	},read_from(behaviour_));

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
