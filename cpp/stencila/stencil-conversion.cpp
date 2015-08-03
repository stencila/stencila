#include <iostream>

#include <boost/filesystem.hpp> 

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>
#include <stencila/helpers.hpp>

namespace Stencila {

Stencil& Stencil::docx(const std::string& direction, const std::string& path) {
	if(direction!="to") STENCILA_THROW(Exception,"Conversion direction not yet implemented.\n  direction: "+direction);
	
	// Serve this stencil so theme CSS and JS is available
	Component::classes();
	auto url = serve();
	// Convert to PDF using PhantomJS
	auto script = Helpers::script("stencil-docx-phantom.js",R"(

		var page = require('webpage').create();
		var fs = require('fs');
		var args = require('system').args;
		var url = args[1];
		var html = args[2];
		
		// Reusable function for exiting with workaround for bud in PhantonJS 1.9.8 
		// https://github.com/ariya/phantomjs/issues/12697
		function exit(code){
			setTimeout(function(){ phantom.exit(code||0); }, 0);
		}
		// Reusable function for priting error within `page.evaluate` and exiting
		page.onError = function(msg, trace) {
			var msgStack = ['Error: '+msg];
			if(trace && trace.length) {
				trace.forEach(function(t){
					msgStack.push('  ' + t.file + ': ' + t.line + (t.function ? ' (in function "' + t.function +'")' : ''));
				});
			}
			console.error(msgStack.join('\n'));
			exit(1);
		};
		// Reusable callback to capture console messages in `page.evaluate`
		page.onConsoleMessage = function (message) {
			console.log(message);
		};
		// Callback for `page.evaluate` to notify PhantomJS when it is finished.
		page.onCallback = function(message){
			console.log(message);
			finish();
		};
		page.open(url, function(status){
			if(status!=='success') {
				console.error('Error: page could not be opened:\n  '+url);
				exit(1);
			}
			// Wait for page to render
			console.log('Waiting: page to render');
			setTimeout(function(){
				page.evaluate(function(){
					// Convert each <script> math element to MathML and insert
					// it into the DOM
					// See http://docs.mathjax.org/en/latest/toMathML.html
					function getMathML(jax,callback) {
						var mml;
						try {
							mml = jax.root.toMathML('');
						} catch(err) {
							if(!err.restart) {throw err;}
							return MathJax.Callback.After([getMathML,jax,callback],err.restart);
						}
						MathJax.Callback(callback)(jax,mml);
					}
					var jaxesDone = 0;
					function insertMathML(jax,mml){
						var script = $('#'+jax.inputID);
						script.after(mml);
						jaxesDone += 1;
						if(jaxesDone===jaxes.length) window.callPhantom('Done : '+jaxesDone+' MathJax jaxes');
					}
					// Convert all jaxes and notify PhantomJS when done
					console.log('Doing: MathJax to MathML');
					var jaxes = MathJax.Hub.getAllJax('content');
					if(jaxes.length===0) window.callPhantom('Done: no MathJax');
					for(var i=0;i<jaxes.length;i++) getMathML(jaxes[i],insertMathML);
				});
			},10000);
		});
		// When asynchronous toMathML has finished...
		function finish(){
			// Remove HTML elements from page
			//	- MathJax displays and errors 
			//	- [data-exec],[data-off] (the equivalent of the C++ method `crush()`)
			//	- #title because pandoc uses <title>
			//	- script elements in the content (which break lines in Word)
			//	- any body childen that are not content (menu, other MathJax elements)
			//	- head styles and scripts (just because they are unecssary and quite large)
			console.log('Doing: clean up');
			page.evaluate(function(){
				$('[class^=MathJax], [data-exec], [data-off], #title, #content script, body>:not(#content), head style, head script').remove();
			});
			// Write to file and exit
			fs.write(html, page.content, 'w');
			exit();
		}
	)");
	auto html = Host::temp_filename("html");
	Helpers::execute("phantomjs "+script+" "+url+" "+html);
	// Convert HTML to DOCX using pandoc
	Helpers::execute("pandoc --from html --to docx --output "+path+" "+html);
	
	return *this;
}

Stencil& Stencil::markdown(const std::string& direction, const std::string& path) {
	if(direction!="from") STENCILA_THROW(Exception,"Conversion direction not yet implemented.\n  direction: "+direction);

	auto html = Host::temp_filename("html");
	Helpers::execute("pandoc --from markdown --to html --output "+html+" "+path);
	import(html);

	return *this;
}

Stencil& Stencil::pdf(const std::string& direction, const std::string& path,const std::string& format,const std::string& orientation,const std::string& margin) {
	if(direction!="to") STENCILA_THROW(Exception,"Conversion direction not yet implemented.\n  direction: "+direction);

	// Serve this stencil so theme CSS and JS is available
	Component::classes();
	auto url = serve();
	// Convert to PDF using PhantomJS
	// See https://github.com/adjust/shrimp/blob/master/lib/shrimp/rasterize.js for 
	// a similar application of PhantomJS with more options
	auto script = Helpers::script("stencil-pdf-phantom.js",R"(
		var page = require('webpage').create();
		var args = require('system').args;
		var url = args[1];
		var pdf = args[2];
		var format = args[3];
		var orientation = args[4];
		var margin = args[5];

		page.paperSize = {
			format: format,
			orientation: orientation,
			margin: margin
		};

		page.open(url, function(){
			// Wait for page to render
			var renderTime = 10000;
			setTimeout(function(){
				page.render(pdf);
				phantom.exit();
			},renderTime);
		});
	)");
	Helpers::execute("phantomjs '"+script+"' '"+url+"' '"+path+"' '"+format+"' '"+orientation+"' '"+margin+"'");

	return *this;
}

Stencil& Stencil::compile(void){
	render();
	auto home = boost::filesystem::path(path(true));
	export_((home/"page.html").string());
	preview((home/"preview.png").string());
	return *this;
}

}
