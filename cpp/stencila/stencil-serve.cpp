#include <cstdlib>

#include <boost/filesystem.hpp>

#include <stencila/component-page.hpp>
#include <stencila/json.hpp>
#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::serve(void){
	return Component::serve(StencilType);
}

Stencil& Stencil::view(void){
	Component::view(StencilType);
	return *this;
}

Stencil& Stencil::preview(const std::string& path){
	Component::preview(StencilType,path);
	return *this;
}

std::string Stencil::page(void) const {
	// Get base document
	Html::Document doc = Component_page_doc<Stencil>(*this);
	Html::Node head = doc.find("head");
	Html::Node body = doc.find("body");

	// Extra metadata
	head.append("meta",{
		{"itemprop","mode"},
		{"content",mode()}
	});
	head.append("meta",{
		{"itemprop","contexts"},
		{"content",join(contexts(),",")}
	});

	// Add stencil content to the main element and give #content id
	auto main = body.select("main");
	main.attr("id","content");
	main.append(*this);

	return doc.dump(false);
}

Stencil& Stencil::page(const std::string& filename) {
	write_to(filename, page());
	return *this;
}

std::string Stencil::request(const std::string& verb,const std::string& method,const std::string& body){
	Json::Document request;
	if(verb=="GET"){
		// FIXME
		// Need to parse out request queries
	}
	else if(body.length()){
		request.load(body);
	}
	Json::Document response = Json::Object();
	std::string signature = verb + " " + method;
	
	// FIXME
	// This is currently a POST but should be changed to a GET
	// after fix above is done
	if(signature=="POST content"){
		auto format = request["format"].as<std::string>();
		auto pretty = request["pretty"].as<bool>();

		std::string content;
		if(format=="html" or format=="") content = html(false,pretty);
		else if(format=="cila") content = cila();
		else {
			response.append("error","format is not 'cila' or 'html'");
		}
		response.append("format",format);
		response.append("content",content);
	}
	else if(signature=="PUT content"){
		auto format = request["format"].as<std::string>();
		auto content = request["content"].as<std::string>();

		html(content).write();
	}
	else if(signature=="PUT render"){
		auto format = request["format"].as<std::string>();
		auto content = request["content"].as<std::string>();
		if(content.length()){
			if(format=="html") html(content);
			else if(format=="cila") cila(content);
			else {
				response.append("error","format is not 'cila' or 'html'");
			}
		}
		
		render();

		response.append("format","html");
		response.append("content",html());
	}
	else if(signature=="PUT boot"){
		response = Component::call("boot","{}");
	}
	else if(signature=="PUT write"){
		write();
	}
	else if(signature=="PUT store"){
		store();
	}
	else if(signature=="PUT restore"){
		restore();
	}
	else {
		throw RequestInvalidException();
	}

	return response.dump();
}

std::string Stencil::interact(const std::string& code){
	if(context_){
		// Switch to stencil's directory
		boost::filesystem::path cwd = boost::filesystem::current_path();
		boost::filesystem::path path = boost::filesystem::path(Component::path(true)); 
		boost::filesystem::current_path(path);
		// Create a new unique id
		static char chars[] = {
			'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z',
			'A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
			'0','1','2','3','4','5','6','7','8','9'
		};
		std::string id;
		for(int cha=0;cha<8;cha++) id += chars[int(std::rand()/double(RAND_MAX)*sizeof(chars))];
		// Run code in context
		auto result = context_->interact(code,id);
		// Return to original working directory
		boost::filesystem::current_path(cwd);
		return result;
	} else {
		STENCILA_THROW(Exception,"No context attached to this stencil");
	}
}

Json::Document Stencil::call(const std::string& name, const Json::Document& args) {
	// TODO Apply new API here
	
	#if 0
	auto what = call.what();
	
	// Getting content
	if(what=="html():string"){
		return html();
	}
	else if(what=="cila():string"){
		return cila();
	}

	// Setting content
	else if(what=="html(string)"){
		std::string string = call.arg(0);
		           html(string);
	}
	else if(what=="cila(string)"){
		std::string string = call.arg(0);
		           cila(string);
	}

	// Patching content
	else if(what=="patch(string)"){
		std::string string = call.arg(0);
		           patch(string);	
	}

	// Saving
	else if(what=="html(string).write()"){
		std::string string = call.arg(0);
		           html(string).write();
	}
	else if(what=="cila(string).write()"){
		std::string string = call.arg(0);
		           cila(string).write();
	}

	// Conversion of content...
	// ... HTML to Cila
	else if(what=="html(string).cila():string"){
		std::string string = call.arg(0);
		return     html(string).cila();
	}
	// ... Cila to HTML
	else if(what=="cila(string).html():string"){
		std::string string = call.arg(0);
		return     cila(string).html();
	}

	// Rendering HTML
	else if(what=="html(string).render().html():string"){
		std::string string = call.arg(0);
		return     html(string).render().html();
	}
	else if(what=="html(string).refresh().html():string"){
		std::string string = call.arg(0);
		return     html(string).refresh().html();
	}

	// Rendering Cila
	else if(what=="cila(string).render().cila():string"){
		std::string string = call.arg(0);
		return     cila(string).render().cila();
	}

	// Update <input>s
	else if(what=="inputs({string,string}).render().html():string"){
		auto values = call.arg<std::map<std::string,std::string>>(0);
		return     inputs(     values    ).render().html();
	}
	
	// Restart
	else if(what=="restart().html():string"){
		return     restart().html();
	}    

	// Access to context
	else if(what=="interact(string):string"){
		std::string string = call.arg(0);
		return     interact(string);
	}

	else return Component::call(call);
	#endif

	return "";
}

}
