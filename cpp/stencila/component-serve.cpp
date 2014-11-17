#include <boost/format.hpp>
#include <boost/algorithm/string/replace.hpp>

#include <stencila/component.hpp>
#include <stencila/network.hpp>
#include <stencila/json.hpp>

namespace Stencila {

std::string Component::serve(Type type){
    hold(type);
    return Server::startup() + "/" + address();
}

void Component::view(Type type){
	std::string url = serve(type);
	#if defined(_WIN32) || defined(_WIN64)
	   ShellExecute(NULL, "open", url.c_str(), NULL, NULL, SW_SHOWNORMAL);
	#elif __APPLE__
		std::system(("open \""+url+"\"").c_str());
	#elif __linux
		// Open using xdg-open with all output redirected to null device
		std::system(("2>/dev/null 1>&2 xdg-open \""+url+"\"").c_str());
	#endif
}

std::string Component::page(const std::string& address){
	Instance instance = get(address);
	if(not instance.exists()) return "<html><head><title>Error</title></head><body>No component at address \""+address+"\"</body></html>";
	else return call(instance,&Class::pageing);
}

std::string Component::page(const Component* component){
	return "";
}

std::string Component::page(const Component* component,const std::string& title,const std::string& theme) {
	using boost::format;
	return str(format(R"(
		<html>
			<head>
				<title>%s</title>
				<link rel="stylesheet" type="text/css" href="/%s/theme.css" />
			</head>
			<body>
				<script src="/core/themes/boot.js"></script>
				<script src="/%s/theme.js"></script>
			</body>
		</html>
	)") % title % theme % theme);
}

std::string Component::message(const std::string& address,const std::string& message){
	using boost::format;

	//WAMP basic spec is at https://github.com/tavendo/WAMP/blob/master/spec/basic.md
	
	// WAMP message codes used below.
	// From https://github.com/tavendo/WAMP/blob/master/spec/basic.md#message-codes-and-direction
	//static const int ERROR = 8;
	static const int CALL = 48;
	static const int RESULT = 50;
	//static const int YIELD = 70;

	//[ERROR, REQUEST.Type|int, REQUEST.Request|id, Details|dict, Error|uri]
	//[ERROR, REQUEST.Type|int, REQUEST.Request|id, Details|dict, Error|uri, Arguments|list]
	//[ERROR, REQUEST.Type|int, REQUEST.Request|id, Details|dict, Error|uri, Arguments|list, ArgumentsKw|dict]

	try {
		Instance instance = get(address);
		if(not instance.exists()){
			return "[8, 0, 0, {}, \"no component at address\", [\"" + address + "\"]]";
		} else {

			Json::Document request(message);

			int items = request.size();
			if(items<1) STENCILA_THROW(Exception,"Malformed message");

			char code = request[0].as<int>();
			if(code==CALL){
				//[CALL, Request|id, Options|dict, Procedure|uri]
				//[CALL, Request|id, Options|dict, Procedure|uri, Arguments|list]
				//[CALL, Request|id, Options|dict, Procedure|uri, Arguments|list, ArgumentsKw|dict]
				
				if(items<2) STENCILA_THROW(Exception,"Malformed message");
				int id = request[1].as<int>();
				
				if(items<4) STENCILA_THROW(Exception,"Malformed message");
				std::string procedure = request[3].as<std::string>();

				std::vector<std::string> args;
				if(items>=5) args = request[4].as<std::vector<std::string>>();

				std::map<std::string,std::string> kwargs;
				if(items>=6) kwargs = request[5].as<std::map<std::string,std::string>>();
				
				std::string result;
				try {
					Call call(procedure,args,kwargs);
					result = Component::call(instance,&Class::calling,call);
				}
				catch(const std::exception& e){
					std::string message = e.what();
					// Escape quotes to prevent JSON parsing errors
					boost::replace_all(message,"\"","\\\"");
					return str(format("[8, 48, %d, {}, \"%s\"]")%id%message);
				}
				catch(...){
					return str(format("[8, 48, %d, {}, \"unknown exception\"]")%id);         
				}

				//[RESULT, CALL.Request|id, Details|dict]
				//[RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
				//[RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list, YIELD.ArgumentsKw|dict]
				Json::Document response = Json::Array();
				response.append(RESULT);
				response.append(id);                                // CALL.Request|id
				response.append(Json::Object());                    // Details|dict
				std::vector<std::string> yield_args = {result};
				response.append(yield_args);                        // YIELD.Arguments|list
				return response.dump();
			}
			return "[8, 0 , 0,{},\"unhandle message code\"]";
		}
	}
	// Most exceptions should be caught above and WAMP ERROR messages appropriate to the 
	// request type returned. The following are failovers if that does not happen...
	catch(const std::exception& e){
		std::string message = e.what();
		// Escape quotes to prevent JSON parsing errors
		boost::replace_all(message,"\"","\\\"");
		return std::string("[8, 0, 0, {}, \"") + message + "\"]";
	}
	catch(...){
		return "[8, 0, 0, {}, \"unknown exception\"]";         
	}
	// This exception is intended to capture errors in coding above where none of the branches
	// return a string
	STENCILA_THROW(Exception,"Implementation error; message not handles properly");
}

std::string Component::message(Component* component, const std::string& message){
	return "{}";
}

std::string Component::home(void){
	Html::Document page(R"(
		<html>
			<head>
				<title>Stencila</title>
			</head>
			<body></body>
		</html>
	)");
	auto list = page.select("body").append("ul");
	for(auto instance : instances_){
		list.append("li",instance.first);
	}
	return page.dump();
}

}
