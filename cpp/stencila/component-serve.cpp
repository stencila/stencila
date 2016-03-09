#include <boost/format.hpp>
#include <boost/algorithm/string/replace.hpp>

#include <stencila/debug.hpp>
#include <stencila/component.hpp>
#include <stencila/network.hpp>
#include <stencila/json.hpp>
#include <stencila/wamp.hpp>

namespace Stencila {

std::string Component::serve(Type type){
	hold(type);
	// URL should include trailing slash to avoid redictions
	// and provide proper serving
	return Server::startup() + "/" + address() + "/";
}

Component& Component::view(Type type){
	std::string url = serve(type);
	#if defined(_WIN32) || defined(_WIN64)
	   ShellExecute(NULL, "open", url.c_str(), NULL, NULL, SW_SHOWNORMAL);
	#elif __APPLE__
		std::system(("open \""+url+"\"").c_str());
	#elif __linux
		// Open using xdg-open with all output redirected to null device
		std::system(("2>/dev/null 1>&2 xdg-open \""+url+"\"").c_str());
	#endif
	return *this;
}

Component& Component::preview(Type type, const std::string& path) {
	// Serve this component so that theme CSS and JS is available
	auto url = serve(type)+"#preview!";
	// Convert to PNG using PhantomJS
	auto script = Helpers::script("component-preview-phantom.js",R"(
		var page = require('webpage').create();
		var args = require('system').args;
		var url = args[1];
		var png = args[2];

		page.open(url, function(){
			// Wait for page to render
			var renderTime = 5000;
			setTimeout(function(){
				var clip = page.evaluate(function(){
					var target;
					target = document.querySelector('#preview');
					if(target) return target.getBoundingClientRect();
					else return null;
				});
				if(clip){
					// Clip the page to the target 
					page.clipRect = clip;
				} else {
					// Use a viewportSize that is what is
					// wanted for final preview. Adjust zoomFactor
					// to tradeoff extent/clarity of preview
					page.viewportSize = { width: 480, height: 300 };
					page.zoomFactor = 0.5;
				}
				page.render(png);
				phantom.exit();
			},renderTime);
		});
	)");
	auto temp = Host::temp_filename("png");
	Helpers::execute("phantomjs '"+script+"' '"+url+"' '"+temp+"'");
	// Resize is necessary because viewportSize only seems to be relevant to width
	// and not to height
	Helpers::execute("convert "+temp+" -crop '480x300+0+0' "+path);
	return *this;
}

std::string Component::page_dispatch(const std::string& address){
	Instance instance = get(address);
	if(not instance.exists()){
		return "<html><head><title>Error</title></head><body>No component at address \""+address+"\"</body></html>";
	}
	else {
		auto method = Class::get(instance.type()).page_method;
		if (method) {
			return method(instance);
		} else {
			throw MethodUndefinedException("page", instance, __FILE__, __LINE__);
		}
	}
}

std::string Component::request_dispatch(const std::string& address, const std::string& verb, const std::string& name, const std::string& body){
	Instance instance = get(address);
	if(not instance.exists()) {
		return "404";
	} else {
		auto method = Class::get(instance.type()).request_method;
		if (method) {
			return method(instance, verb, name, body);
		} else {
			throw MethodUndefinedException("request", instance, __FILE__, __LINE__);
		}
	}
}

std::string Component::message_dispatch(const std::string& address, const std::string& message) {
	Instance instance = get(address);
	if(not instance.exists()) {
		return "404";
	} else {
		auto method = Class::get(instance.type()).message_method;
		if (method) {
			return method(instance, message);
		} else {
			throw MethodUndefinedException("message", instance, __FILE__, __LINE__);
		}
	}
}

std::string Component::page(void) {
	return "";
}

std::string Component::request(const std::string& verb, const std::string& name, const std::string& body) {
	std::function<Json::Document(const std::string&, const Json::Document&)> callback = [&](const std::string& name, const Json::Document& args){
		return this->call(name, args);
	};
	return Component::request(verb, name, body, &callback);
}

std::string Component::request(
	const std::string& verb, const std::string& name, const std::string& body,
	std::function<Json::Document(const std::string&, const Json::Document&)>* callback
) {
    Json::Document args;
    if (body.length()) {
    	args.load(body);
    }
    Json::Document response;
    try {
        response = (*callback)(name, args);
    } catch (const std::exception& exc) {
        response.append("error", exc.what());
    } catch (...) {
        response.append("error", "Unknown exception");
    }
    return response.dump();
}

std::string Component::message(const std::string& message) {
	std::function<Json::Document(const std::string&, const Json::Document&)> callback = [&](const std::string& name, const Json::Document& args){
		return this->call(name, args);
	};
	return Component::message(message, &callback);
}

std::string Component::message(const std::string& message, std::function<Json::Document(const std::string&, const Json::Document&)>* callback) {
    Wamp::Message response;
    Wamp::Call call(message);
    try {
        auto name = call.procedure();
        auto args = call.args();
        response = call.result(
            (*callback)(name, args)
        );
    } catch (const std::exception& exc) {
        response = call.error(exc.what());
    } catch (...) {
        response = call.error("Unknown exception");
    }
    return response.dump();
}

Json::Document Component::call(const std::string& name, const Json::Document& args) {
	Json::Document result;
	if(name=="boot") {
		result.append("rights", "ALL");
		Json::Document session = Json::Object();
		session.append("local", true);
		session.append("websocket", "ws://localhost:7373/" + address());
		result.append("session", session);
	} else if (name=="commit") {
		auto message = args[0].as<std::string>();
		auto id = commit(message);
		result.append("id", id);
	} else {
        STENCILA_THROW(Exception, "Unhandled method name.\n  name: " + name); 
    }
    return result;
}

std::string Component::index(void){
	Html::Document page(R"(
		<html>
			<head>
				<title>Stencila</title>
			</head>
			<body></body>
		</html>
	)");
	auto ul = page.select("body").append("ul");
	for(auto instance : instances_){
		auto li = ul.append("li");
		li.append("span",{{"class","type"}},type_name(instance.second.type()));
		li.append("a",{{"href","./"+instance.first}},instance.first);
	}
	return page.dump();
}

std::string Component::extras(void){
	Xml::Document page(R"(
		<div>
			<div id="styles">
			</div>
			<div id="prepend">
				<header id="header">
				</header>
			</div>
			<div id="append">
			</div>
		</div>
	)");
	return page.dump();
}

}
