#include <pugixml.hpp>

#include <stencila/xml.hpp>

namespace Stencila {
namespace Xml {

Node& Node::patch(const Node& patch){
	// Quoted comments are from https://tools.ietf.org/html/rfc5261
	// Implementation assisted by https://github.com/urho3d/Urho3D/blob/1c4e6f43ff6917403d482f9cfa195c6bde77be9b/Source/Urho3D/Resource/XMLFile.cpp#L189
    for(Node operation : patch.children()) {
		// "Each patch operation element contains a 'sel' attribute.  The value
		// of this attribute is an XPath selector with a restricted subset of
		// the full XPath 1.0 recommendation.  The 'sel' value is used to locate
		// a single unique target node from the target XML document."
   		auto selector = operation.attr("sel");
        if(not selector.length()) STENCILA_THROW(Exception, "Patch operation is missing `sel` attribute for selector");

        auto target = pimpl_->select_single_node(selector.c_str());
        if(not target) STENCILA_THROW(Exception, "Selector is not valid");
        Node target_node = target.node();
        auto target_attr = target.attribute();

        auto name = operation.name();
        if(name=="add"){
        	// Check that not trying to add to an attribute
        	if(target_attr) STENCILA_THROW(Exception,"Attempting to use the add operation on an element.");
		    // "The value of the optional 'type' attribute is only used when adding attributes and namespaces"
		    auto type = operation.attr("type");
		    if(type.length()){
		        // Adding an attribute
		        if(type[0]=='@'){
	    			auto name = type.substr(1);
	    			auto value = operation.text();
				    target_node.attr(name,value);
				}
				// Adding a namespace
				else STENCILA_THROW(Exception,"Adding of namespaces is not supported.");
		    }
		    // Adding elements
		    else {
		    	// "The value of the optional 'pos' attribute indicates the positioning of new data content"
   				// Position defaults to append
		    	auto pos = operation.attr("pos");
    			if(pos.length()==0 or pos=="append"){
    				target_node.append_children(operation);
    			}
    			else if(pos=="prepend"){
    				target_node.prepend_children(operation);
    			}
    			else if(pos=="before"){
    				for(auto child:operation.children()){
    					target_node.before(child);
    				}
    			}
    			else if(pos=="after"){
    				auto previous = target_node;
    				for(auto child:operation.children()){
    					previous = previous.after(child);
    				}
    			}
    			else STENCILA_THROW(Exception,"Unhandled add pathc position.\n  position: "+pos);
		    }
        }
        else if(name=="replace"){
			// Replacing element
			if(target_node and not target_attr){
			    target_node.before(operation.first());
			    target_node.destroy();
			}
			// Replacing attribute
			else if(target_attr){
				target_attr.set_value(operation.text().c_str());
			}
        }
        else if(name=="remove"){
			// Removing element
			if(target_node and not target_attr){
			    target_node.destroy();
			}
			// Removing attribute
			else if(target_attr){
				target.parent().remove_attribute(target_attr);
			}
        }
        else STENCILA_THROW(Exception,"Patch operation element name should be one of 'add', 'replace' or 'remove'.\n  name: "+name);
    }
	return *this;
}

Node& Node::patch(const std::string& patch_string){
	Document doc(patch_string);
	return patch(doc);
	return *this;
}

}  // namesace Xml
}  // namespace Stencila
