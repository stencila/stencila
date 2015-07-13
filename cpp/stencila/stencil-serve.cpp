#include <cstdlib>

#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>
#include <stencila/component-page.hpp>

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

std::string Stencil::page(const Component* component){
	return static_cast<const Stencil&>(*component).page();
}

std::string Stencil::page(void) const {
	// Get base document
	Html::Document doc = Component_page_doc<Stencil>(*this);
	Html::Node head = doc.find("head");
	Html::Node body = doc.find("body");

	// Extra metadata
	head.append("meta",{
		{"itemprop","closed"},
		{"content",closed()?"true":"false"}
	});
	head.append("meta",{
		{"itemprop","contexts"},
		{"content",join(contexts(),",")}
	});

	// Create a sanitized copy of the stencil to insert into the page
	// (this is a const function; should not alter this stencil itself)
	Stencil copy;
	copy.sanitize();

	// Content is placed in a <main> rather than just using the <body> so that 
	// extra HTML elements can be added by the theme without affecting the stencil's content.
	// Note that this is prepended to body so it is before launch script
	auto content = body.prepend("main",{
		{"id","content"}
	}," ");
	content.append(copy);

	return doc.dump();
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

std::string Stencil::call(Component* component, const Call& call){
	return static_cast<Stencil&>(*component).call(call);
}

std::string Stencil::call(const Call& call) {
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

	return "";
}

}
