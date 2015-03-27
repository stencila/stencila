#include <boost/filesystem.hpp>

#include <stencila/theme.hpp>
#include <stencila/stencil.hpp>
#include <stencila/helpers.hpp>
#include <stencila/map-context.hpp>

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
			style(filename.string());
			break;
		}
	}
	for(std::string file : {"theme.js"}){
		boost::filesystem::path filename = boost::filesystem::path(path()) / file;
		if(boost::filesystem::exists(filename)){
			behaviour(filename.string());
			break;
		}
	}
	return *this;
}

Theme& Theme::compile(void) {
	if(style_.length()){
		// Convert CSS or SCSS to compressed CSS using SASS
		auto script = Helpers::script("theme-make-mincss.js",R"(
			var sass = require('node-sass');
			var fs = require('fs');
			var args = process.argv.slice(2); // Remove "node" and <script name> args

			var from = args[0];
			var to = 'theme.min.css';

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
		Helpers::execute("node '"+script+"' '"+style_+"'");
	}
	if(behaviour_.length()){
		// Convert JS to compressed JS using UglifyJS
		Helpers::execute("uglifyjs '"+behaviour_+"' -m > theme.min.js");
	}
	return *this;
}


std::string Theme::serve(void){
	return Component::serve(ThemeType);
}

void Theme::view(void){
	return Component::view(ThemeType);
}

std::string Theme::page(const Component* component){
	return static_cast<const Theme&>(*component).page();
}

std::string Theme::page(void) const {
	// Create a stencil to reflect the theme in HTML
	// We could build this using a `Html::Document` but the
	// stencil provides templating convienience and adds all
	// the right `<style>` and `<script>` elements based on the 
	// theme.
	//! @todo This currently is pretty basic 
	Stencil page(R"(cila://
		#title Theme
		#theme core/themes/themes/default

		- Style : ``style``
		- Behaviour : ``behaviour``
	)");
	auto& map = *new MapContext;
	map.assign("style",style_);
	map.assign("behaviour",behaviour_);
	page.render(&map);

	// Return HTML document with no indentation
	return page.html(true,false);
}

std::string Theme::call(Component* component, const Call& call){
	return static_cast<Theme&>(*component).call(call);
}

std::string Theme::call(const Call& call) {
	auto what = call.what();
	return "";
}

}
