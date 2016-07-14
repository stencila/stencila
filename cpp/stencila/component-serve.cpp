#include <boost/format.hpp>
#include <boost/algorithm/string/replace.hpp>

#include <stencila/debug.hpp>
#include <stencila/component.hpp>
#include <stencila/network.hpp>
#include <stencila/json.hpp>
#include <stencila/wamp.hpp>

namespace Stencila {

std::string Component::serve(Type type){
	// Hold this component
	hold(type);
	// Start server and return URL for this component including trailing
	// slash to avoid redictions and provide for relative URLs in links
	return Server::startup().url("http",address()+"/");
}

Component& Component::view(Type type){
	std::string url = serve(type);
	int result = 0;
	#if defined(_WIN32) || defined(_WIN64)
	   ShellExecute(NULL, "open", url.c_str(), NULL, NULL, SW_SHOWNORMAL);
	#elif __APPLE__
		result = std::system(("open \""+url+"\"").c_str());
	#elif __linux
		// Open using xdg-open with all output redirected to null device
		result = std::system(("2>/dev/null 1>&2 xdg-open \""+url+"\"").c_str());
	#endif
	if (result != 0) STENCILA_THROW(Exception, "Error opening URL.\n  url: " + url);
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

std::string Component::message_dispatch(const std::string& message, int connection) {
	Wamp::Message request(message);
	Instance instance = get(request.procedure_address());
	if(not instance.exists()) {
		return "404";
	} else {
		Wamp::Message response;
		try {
			switch (request.type()) {
				case request.CALL: {
					auto method = Class::get(instance.type()).message_method;
					if (method) {
						response = method(instance, request);
					} else {
						throw MethodUndefinedException("message", instance, __FILE__, __LINE__);
					}
				} break;

				case request.SUBSCRIBE: {
					subscribers_[instance.pointer()].push_back(connection);
					response = Wamp::Message::subscribed(request.request(), connection);
				} break;

				default:
					STENCILA_THROW(Exception, "Unhandled message type\n  type: " + string(request.type()));
				break;
			}
		} catch (const std::exception& exc) {
			response = request.error(exc.what());
		} catch (...) {
			response = request.error("Unknown exception");
		}
		return response.dump();
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

Wamp::Message Component::message(const Wamp::Message& message) {
	return message.result(
		call(
			message.procedure_method(), 
			message.args()
		)
	);
}

Wamp::Message Component::message(const Wamp::Message& message, std::function<Json::Document(const std::string&, const Json::Document&)>* callback) {
	return message.result(
		(*callback)(
			message.procedure_method(), 
			message.args()
		)
	);
}

const Component& Component::notify(const Json::Document& event) const {
	auto subscribers = subscribers_[this];
	if (subscribers.size()) {
		auto wamp = Wamp::Message::event(event);
		auto json = wamp.dump();
		auto& server = Server::instance();
		for (auto subscriber : subscribers) {
			server.send(subscriber, json);
		}
	}
	return *this;
}

std::map<const Component* const, std::vector<int> > Component::subscribers_;

Json::Document Component::call(const std::string& name, const Json::Document& args) {
	Json::Document result;
	if(name=="boot") {
		result.append("rights", "ALL");
		Json::Document session = Json::Object();
		// Indicate a local session
		session.append("local", true);
		// Return empty string for websocket URL to indicate that the 
		// client should construct the websocket URL from the hostname of the 
		// window. We do this because, from here, we can't find out what address this
		// request was made on. It won't necessarily be localhost.
		session.append("websocket", "");
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
		li.append("span",{{"class","type"}},type_to_string(instance.second.type()));
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
