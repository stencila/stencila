#include <stencila/component.hpp>
#include <stencila/network.hpp>

namespace Stencila {

std::string Component::serve(Type type){
    hold(type);
    return Server::ensure() + "/" + address();
}

void Component::view(Type type){
	std::string url = serve(type);
	#if defined(_WIN32) || defined(_WIN64)
	   ShellExecute(NULL, "open", url, NULL, NULL, SW_SHOWNORMAL);
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
	using Json::size;
	using Json::as;

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

			int items = size(request);
			if(items<1) STENCILA_THROW(Exception,"Malformed message");

			char code = as<int>(request[0]);
			if(code==CALL){
				//[CALL, Request|id, Options|dict, Procedure|uri]
				//[CALL, Request|id, Options|dict, Procedure|uri, Arguments|list]
				//[CALL, Request|id, Options|dict, Procedure|uri, Arguments|list, ArgumentsKw|dict]
				
				if(items<2) STENCILA_THROW(Exception,"Malformed message");
				int id = as<int>(request[1]);
				
				if(items<4) STENCILA_THROW(Exception,"Malformed message");
				std::string procedure = as<std::string>(request[3]);

				std::vector<std::string> args;
				if(items>=5){
					Json::Value& args_value = request[4];
					args.resize(size(args_value));
					for(uint i=0;i<args.size();i++) args[i] = as<std::string>(args_value[i]);
				}

				std::map<std::string,std::string> kwargs;
				if(items>=6){
					/**
					 * @fixme Not implemented
					 */
					#if 0
					Json::Value& kwargs_value = request[5];
					for(int i=0;i<size(kwargs_value);i++){
						auto value = kwargs_value[i];
						auto name = 
						args[name] = value;
					}
					#endif
				}
				
				std::string result;
				try {
					Call call(procedure,args,kwargs);
					result = Component::call(instance,&Class::calling,call);
				}
				catch(const std::exception& e){
					return str(format("[8, 48, %d, {}, \"%s\"]")%id%e.what());
				}
				catch(...){
					return str(format("[8, 48, %d, {}, \"unknown exception\"]")%id);         
				}

				//[RESULT, CALL.Request|id, Details|dict]
				//[RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
				//[RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list, YIELD.ArgumentsKw|dict]
				Json::Document response;
				response.type<Json::Array>()
						.push(RESULT)
						.push(id)                                // CALL.Request|id
						.push(Json::Object())                          // Details|dict
						.push(std::vector<std::string>{result}); // YIELD.Arguments|list
				return response.dump();
			}
			return "[8, 0 , 0,{},\"unhandle message code\"]";
		}
	}
	// Most exceptions should be caught above and WAMP ERROR messages appropriate to the 
	// request type returned. The following are failovers if that does not happen...
	catch(const std::exception& e){
		return std::string("[8, 0, 0, {}, \"") + e.what() + "\"]";
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
