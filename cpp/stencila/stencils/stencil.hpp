//! @file stencil.hpp
//! @brief Definition of class Stencil
//! @author Nokome Bentley

#pragma once

#include <string>
#include <fstream>

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/join.hpp>
#include <boost/filesystem.hpp>

#include <stencila/component.hpp>
#include <stencila/html.hpp>
#include <stencila/json.hpp>

namespace Stencila {
namespace Stencils {

namespace Stem {
    void parse(const std::string&, Xml::Node);
    std::string print(const std::string& stem);
}

//! [Polyglot markup](http://www.w3.org/TR/html-polyglot/) is both HTML5 and XML. Some people call it XHTML5
//! There is a good summary of what XHTML5 requires [here](http://blog.whatwg.org/xhtml5-in-a-nutshell).
//! Note that this page should be served with the right MIME type i.e. "Content-Type: application/xhtml+xml" (although this is 
//! not supported by older versions of Microsoft IE (< 8.0?))
class Stencil : public Component<Stencil> {
private:

    std::vector<std::string> languages_;
    Html::Document html_;

public:

    static std::string type(void){
        return "stencil";
    };

    /**
     * @name Constructors
     * @{
     */
    
    Stencil(void){
        from_scratch();
    }
    
    Stencil(const Id& id){
        from_id(id);
    }

    Stencil(const std::string& content){
        from(content);
    }

    /**
     * @}
     */
    
    /**
     * @name Initialisation methods
     *
     * Methods for initialising a Stencil from alternative sources
     */

    Stencil& from(const std::string& content){
        /*
        html://
        stem://
        
        file://
        http://
        
        id://
        find://
        */
        std::size_t found = content.find("://");
        if(found==std::string::npos) STENCILA_THROW(Exception,"Type separator (://) not found")
        std::string type = content.substr(0,found);
        std::string rest = content.substr(found+3);
        if(type=="html") from_html(rest);
        else if(type=="stem") from_stem(rest);
        else if(type=="file") from_file(rest);
        else if(type=="id") from_id(rest);
        else STENCILA_THROW(Exception,"Unrecognised type: " + type)

        return *this;
    }

    //! @brief Create a stencil from scratch
    //!
    //! A XHTML5 document is created with a empty head and body
    //! Not that elements are added to the head element when the stencil is saved (see Stencil::dump)
    //!
    //! @return This stencil
    Stencil& from_scratch(void){
        html_.prepend_doctype_html5();
        Html::Node html = html_.append("html",{{"xmlns","http://www.w3.org/1999/xhtml"}});
        
        Html::Node head = Html::Document::append(html,"head");
       
        Html::Document::append(head,"link",{
            {"rel","stylesheet"},
            {"type","text/css"},
            {"href","http://static.stenci.la/css/stencil-default.css"},
        }); 

        // Note that script elements can not be empty (ie not <script .../> but <script ...></script>)
        // hence the empty content added below
        Html::Document::append(head,"script",{
            {"type","text/javascript"},
            {"src","http://static.stenci.la/js/stencil-default.js"},
        }," ");

        html_.append(html,"body");
        return *this;
    }

    //! @brief 
    //!
    //! Certain elements within the head are parsed into stencil meta-data attributes e.g. meta name="keywords"
    //! Any other elements within the head will be ignored e.g. script, link
    //!
    //! @param html
    //! @return This stencil
    Stencil& from_html(const std::string& html){
        // Tidy HTML and load it into this stencil
        std::string html_tidy = Html::tidy(html);
        html_.load(html_tidy);
        
        //! @todo Extract metadata
        Html::Node head = html_.find("head");
        
        Html::Node keywords = html_.find(head,"meta","name","keywords");
        if(keywords){
            std::string content = Xml::Document::get(keywords,"content").value();
            std::vector<std::string> keywords_items;
            boost::split(keywords_items,content,boost::is_any_of(","));
            for(std::string& keyword : keywords_items) boost::trim(keyword);
            keywords_ = keywords_items;
        }
        
        Html::Node id = html_.find(head,"meta","name","id");
        if(id){
            id_ = Xml::Document::get(id,"content").value();
        }
        
        // Now remove the extisting head and replace it with a new one
        html_.remove(head);
        html_.append(html_.find("html"),"head");
        return *this;
    }

    //! @brief 
    //! @param stem
    //! @return 
    Stencil& from_stem(const std::string& stem){
        from_scratch();
        Stem::parse(stem,html_.find("body"));
        return *this;
    }

    //! @brief 
    //! @param path
    //! @return 
    Stencil& from_file(const std::string& path){
        std::ifstream file(path);
        std::stringstream buffer;
        buffer<<file.rdbuf();
        std::string ext = boost::filesystem::path(path).extension().string();
        if(ext==".html") {
            from_html(buffer.str());
        }
        else if(ext==".stem") {
            from_stem(buffer.str());
        } 
        else {
            STENCILA_THROW(Exception,"File extension not interpreted as a stencil:"+ext)
        }
        
        return *this;
    }

    Stencil& from_id(const Id& id){
        STENCILA_THROW(Unimplemented,"Stencil::from_id");
        return *this;
    }

    /**
     * @}
     */


    /**
     * @name Attribute getters and setters
     * @{
     */

    /**
     * Get the languages that are supported by the stencil
     */
    const std::vector<std::string> languages(void) const {
        return languages_;
    }

    /**
     * Get the languages that are supported by the stencil
     */
    std::vector<std::string> languages(void) {
        return languages_;
    }

    /**
     * Set the languages that are supported by the stencil
     */
    Stencil& languages(const std::vector<std::string>& values) {
        languages_ = values;
        return *this;
    }

    /**
     * @}
     */
    
    
    /**
     * @name Content getters and setters
     *
     * Methods for getting and setting a Stencil's content using strings
     * 
     * @{
     */
    
    std::string content(const std::string& language="html") const {
        if(language=="html") return html();
        else if(language=="stem") return stem();
        else if(language=="inline") return inlin();
        else STENCILA_THROW(Exception,"Unrecognised language code: "+language)
    }

    Stencil& content(const std::string& content, const std::string& language) {
        if(language=="html") return html(content);
        else if(language=="stem") return stem(content);
        else if(language=="inline") return inlin(content);
        else STENCILA_THROW(Exception,"Unrecognised language code: "+language)
    }

    std::string html(void) const {
        std::ostringstream out;
        for(Html::Node child : html_.find("body").children()) child.print(out,"",pugi::format_raw);
        return out.str();
    }
    
    Stencil& html(const std::string& html) {
        Html::Document html_doc(html);
        html_.copy(html_.find("body"),html_doc.find("body"));
        return *this;
    }
    
    Stencil& html_append(const std::string& html){
        Html::Document html_doc(html);
        html_.append(html_.find("body"),html_doc.find("body"));
        return *this;
    }

    std::string stem(void) const {
        STENCILA_THROW(Unimplemented,"Stencil::stem");
        return "";
    }
    
    Stencil& stem(const std::string& stem) {
        STENCILA_THROW(Unimplemented,"Stencil::stem");
        return *this;
    }

    std::string inlin(void) const {
        STENCILA_THROW(Unimplemented,"Stencil::inlin");
        return "";
    }
    
    Stencil& inlin(const std::string& native) {
        STENCILA_THROW(Unimplemented,"Stencil::inlin");
        return *this;
    }

    /**
     * @}
     */
    
    //! @name Serialisation methods
    //! 
    //! Methods for loading from, or dumping to, a string
    //! 
    //! @{
    
    Stencil& load(std::string& html){
        STENCILA_THROW(Unimplemented,"Stencil::load");
        return *this;
    }

    //! @brief Dump the stencil into a string
    //!
    //! Serialise meta-data into head
    //! @return std::string representation of stencil
    std::string dump(void){

        // Construct a XHTML5 document
        Html::Document doc;

        doc.prepend_doctype_html5();

        /**
         * @todo Consider implementing conditional classes on the <html> element
         *
         * See [this](http://htmlcssjavascript.com/html/how-did-the-ie-conditional-classes-get-on-the-html-element-in-html5-boilerplate/)
         * they may not be necessary. HTML5BoilerPlate has removed them
         *
         * 
            <!--[if lt IE 7]> <html class="no-js lt-ie9 lt-ie8 lt-ie7" lang="en"> <![endif]-->
            <!--[if IE 7]> <html class="no-js lt-ie9 lt-ie8" lang="en"> <![endif]-->
            <!--[if IE 8]> <html class="no-js lt-ie9" lang="en"> <![endif]-->
            <!--[if gt IE 8]><!--> <html lang="en"> <!--<![endif]-->
                <head>
                    <script>(function(H){H.className=H.className.replace(/\bno-js\b/,'js')})(document.documentElement)</script>
         */
        auto html = doc.append("html",{
            // The page language should be specified for screen readers since no default language is defined in the spec.
            {"lang","en"},
            // Application cache for offline use
            {"manifest","http://get.stenci.la/stencil.appcache"}
        });

        auto head = doc.append("head");

        /*
        Although it is not technically required to define the character set, failing to do so can leave the page vulnerable to cross-site scripting attacks in older versions of IE. Note that even in old browsers this short version is equivalent to:
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
        (http://www.coreservlets.com/html5-tutorial/basic-html5-document.html)
        */
        doc.append(head,"meta",{
            {"charset","utf-8"}
        },"");

        doc.append(head,"title",title());
        
        doc.append(head,"meta",{
            {"name","id"},
            {"content",id()}
        },"");

        doc.append(head,"meta",{
            {"name","keywords"},
            {"content",boost::algorithm::join(keywords(),", ")}
        },"");

        doc.append(head,"meta",{
            {"name","description"},
            {"content",description()}
        },"");

        /**
         * <link rel="stylesheet" ...
         *
         * Links to CSS stylesheets ate [placed in the head](http://developer.yahoo.com/performance/rules.html#css_top)
         */
        doc.append(head,"link",{
            {"rel","stylesheet"},
            {"type","text/css"},
            {"href","http://get.stenci.la/core/themes/default/base.min.css"}
        },"0;");

        auto body = doc.append(html,"body");

        /**
         * #languages
         */
        auto langs = doc.append(body,"ul",{
            {"id","languages"}
        });
        for(auto lang : languages()){
            doc.append(langs,"li",{
                {"class",lang}
            },lang);
        }

        /**
         * #authors
         *
         * Use both <address> and <a rel="author" ...> as suggested at http://stackoverflow.com/a/7295013 .
         * The placement of <address> as a child of <body> should mean that this authors list applies to the whole document.
         * See:
            http://html5doctor.com/the-address-element/
            http://www.w3.org/TR/html5/sections.html#the-address-element
            http://stackoverflow.com/questions/7290504/which-html5-tag-should-i-use-to-mark-up-an-authors-name
         */
        auto address = doc.append(body,"address",{
            {"id","authors"}
        });
        for(auto author : authors()){
            doc.append(address,"a",{
                {"rel","author"},
                {"href","#"}
            },author);
        }

        /**
         * #content
         *
         * This placed in a <div> rather than just using the <body> so that extra HTML elements can be added by the 
         * theme without affecting the stencil's content
         */
        auto content = doc.append(body,"main",{
            {"id","content"}
        });
        content.append_copy(html_);

        /**
         * <script>
         *
         * Script elements are [placed at bottom of page](http://developer.yahoo.com/performance/rules.html#js_bottom)
         */
        doc.append(body,"script",{
            {"src":"http://get.stenci.la/core/themes/default/base.min.js"}
        },"0;");

        return doc.dump();
    }

    //! @}
    
    //! @name Persistence methods
    //! @{

    /**
     * Read the Stencil from a directory
     * 
     * @param directory Filesystem path to directory
     */
    Stencil& read(const std::string& directory){
        std::ifstream file(directory+"/index.html");
        std::string value((std::istreambuf_iterator<char>(file)),(std::istreambuf_iterator<char>()));
        load(value);
        return *this;
    }

    /**
     * Write the Component to a directory
     * 
     * @param directory Filesystem path to directory
     */
    Stencil& write(const std::string& directory) {
        std::ofstream index(directory+"/index.html");
        index<<dump();
        return *this;
    }
    
    //! @}
    
    
    //! @name REST interface methods
    //! @{
    
    std::string get(void){
        Json::Document out;
        out.add("content",html());
        return out.dump();
    }
    
    std::string put(const std::string& data){
        Json::Document json(data);
        if(json.has("content")) html(json.as<std::string>(json.get("content")));
        return "{}";
    }
    
    //! @}
    
    
    //! @name Rendering and display methods
    //! These methods provide alternative ways of rendering a stencil
    //! @{

public:

    //! @brief Render a stencil into an HTML fragment
    //! @param workspace The workspace in which the stencil will be rendered
    //! @return The stencil
    template<typename Workspace>
    Stencil& render(Workspace& workspace){
        render_element(html_,workspace);
        return *this;
    }
    
private:

    //! @brief 
    //! @param node
    //! @param workspace
    //! @return 
    template<typename Workspace>
    void render_element(Xml::Node node, Workspace& workspace){
        try {
            //Check for handled element tag names
            std::string tag = node.name();
            if(tag=="script") {
                return render_script(node,workspace);
            }
            //For each attribute in this node...
            auto attrs = node.attributes();
            for(auto attr=attrs.begin();attr!=attrs.end();attr++){
                //...get the name and value of attribute
                std::string name = attr->name();
                std::string value = attr->value();
                //...use the name of the attribute to dispatch to another method
                //   Note that return is used so that only the first Stencila "data-xxx" will be 
                //   considered and that directive will determin how/if children nodes are processed
                if(name=="data-text") return render_text(node,workspace,value);
                else if(name=="data-image") return render_image(node,workspace,value);
                else if(name=="data-if") return render_if(node,workspace,value);
                else if(name=="data-switch") return render_switch(node,workspace,value);
                else if(name=="data-for") return render_for(node,workspace,value);
                else if(name=="data-with") return render_with(node,workspace,value);
                else if(name=="data-include") return render_include(node,workspace,value);
            }
            //If return not yet hit then process children of this element
            render_children(node,workspace);
        }
        catch(std::exception& exc){
            Xml::Document::set(node,"data-error",exc.what());
        }
        catch(...){
            Xml::Document::set(node,"data-error","unknown error");
        }
    }
    
    //! @brief 
    //! @param workspace
    template<typename Workspace>
    void render_children(Xml::Node node, Workspace& workspace){
        for(Xml::Node child:node.children()){
            render_element(child,workspace);
        }
    }

    //! @brief Execute a script in the workspace
    //! @param node HTML node being rendered
    //! @param workspace Workspace that the node is being rendered in
    template<typename Workspace>
    void render_script(Xml::Node node, Workspace& workspace){
         std::string code = node.text().as_string();
         workspace.script(code);
    }

    //! @brief 
    //! @param workspace
    //! @param expression
    template<typename Workspace>
    void render_text(Xml::Node node, Workspace& workspace, const std::string& expression){
        std::string text = workspace.text(expression);
        node.text().set(text.c_str());
    }
    
    //! @brief Render an image in the workspace
    //! @param node HTML node being rendered
    //! @param workspace Workspace that the node is being rendered in
    template<typename Workspace>
    void render_image(Xml::Node node, Workspace& workspace, const std::string& type){
        workspace.image_begin(type);
        render_children(node,workspace);
        std::string result = workspace.image_end();
        
        if(type=="svg"){
            Xml::Node svg = Xml::Document(result);
            for(Xml::Node child : svg.children()){
                node.append_copy(child);
            }
        }
    }

    //! @brief 
    //! @param workspace
    //! @param expression    
    template<typename Workspace>
    void render_with(Xml::Node node, Workspace& workspace, const std::string& expression){
        //Enter a new block in the workspace
        workspace.enter(expression);
        //Render all children of node within that new block
        render_children(node,workspace);
        //Exit the block
        workspace.exit();
    }

    //! @brief 
    //! @param workspace
    //! @param expression
    template<typename Workspace>
    void render_if(Xml::Node node, Workspace& workspace, const std::string& expression){
        //Test the expression
        bool result = workspace.test(expression);
        //If test passes, render all children
        if(result){
            render_children(node,workspace);
            Xml::Document::set(node,"data-active","true");
        }
        //If test fails, remove the data-active attribute (if it exists)
        else {
            node.remove_attribute("data-active");
        }
    }

    //! @brief 
    //! @param workspace
    //! @param expression
    template<typename Workspace>
    void render_switch(Xml::Node node, Workspace& workspace, const std::string& expression){
        //Evaluate the expression in the workspace
        workspace.subject(expression);
        //Iterate through children to
        // (a) find first child that has an equal when
        // (b) remove data-active attribute
        Xml::Node active;
        for(Xml::Node child : node.children()){
            child.remove_attribute("data-active");
            Xml::Attribute when = Xml::Document::get(child,"data-value");
            if(when){
                bool equal = workspace.match(when.value());
                if(equal){
                    active = child;
                    break;
                }
            } else if(Xml::Document::get(child,"data-default")){
                active = child;
            }
        }
        if(active){
            //Set as active
            Xml::Document::set(active,"data-active","true");
            //Render it
            render_element(active,workspace);
        }
    }

    //! @brief 
    //! @param workspace
    //! @param value
    template<typename Workspace>
    void render_for(Xml::Node node, Workspace& workspace, const std::string& value){
        // Get the name of item and items
        std::vector<std::string> bits;
        boost::split(bits,value,boost::is_any_of(":"));
        std::string item = bits[0];
        std::string items = bits[1];
        // Initialise the loop
        bool more = workspace.begin(item,items);
        // Get the first child element of this node for replication
        Xml::Node first = node.find_child(Xml::Document::is_element);
        // Delete all other nodes
        for(Xml::Node child : node.children()){
            if(child!=first) node.remove_child(child);
        }
        int count = 1;
        while(more){
            if(count==1){
                //Render the first child
                render_element(first,workspace);
            } else {
                //Create a copy of the first child node
                Xml::Node copy = node.append_copy(first);
                //Render the copy
                render_element(copy,workspace);
            }
            //Ask workspace to step
            more = workspace.step();
            count++;
        }
    }

    //! @brief 
    //! @param workspace
    //! @param identifier
    template<typename Workspace>
    void render_include(Xml::Node node, Workspace& workspace, const std::string& identifier){

        //Remove any existing children that have been included previously
        for(Xml::Node child : node.children()){
            if(Xml::Document::has(child,"data-included")){
                node.remove_child(child);
            }
        }

        // Get the included stencil
        Stencil source(identifier);
        Xml::Document sink;
        // Check to see if a subselection of modes is to be included
        Xml::Attribute select = Xml::Document::get(node,"data-select");
        if(select){
            Xml::Nodes included = source.html_.all(select.value());
            for(auto i=included.begin();i!=included.end();i++) sink.append_copy(i->node());
        }
        //Otherwise include all children
        else {
            for(auto i=source.html_.children().begin();i!=source.html_.children().end();i++) sink.append_copy(*i);
        }
        
        //Apply child modifiers
        std::string modifiers[] = {"replace","before","after","prepend","append"};
        enum {replace=0,before=1,after=2,prepend=3,append=4};
        for(Xml::Node child : node.children()){
            for(unsigned int modifier=0;modifier<5;modifier++){
                std::string attr_name = "data-" + modifiers[modifier];
                Xml::Attribute attr = Xml::Document::get(child,attr_name);
                if(attr){
                    Xml::Nodes targets = sink.all(attr.value());
                    for(auto i=targets.begin();i!=targets.end();i++){
                        Xml::Node target = i->node();
                        Xml::Node copy;
                        switch(modifier){
                            case replace: 
                                copy = sink.insert_copy_before(child,target);
                                sink.remove_child(target);
                            break;
                            
                            case before:
                                copy = sink.insert_copy_before(child,target);
                            break;
                            
                            case after:
                                copy = sink.insert_copy_after(child,target);
                            break;
                            
                            case prepend:
                                copy = target.prepend_copy(child);
                            break;
                            
                            case append:
                                copy = target.append_copy(child);
                            break;
                        }
                        copy.remove_attribute(attr_name.c_str());
                    }
                    break;
                }
            }
        }

        //Append new, included children
        for(Xml::Node child : sink.children()){
            Xml::Document::set(child,"data-included","true");
            node.append_copy(child);
        }

        //Create a new workspace with parameters
        //Determine if there are any node parameters so that we don't create a new workspace block unecessarily
        bool params = Xml::Document::has(node,"data-param");
        if(params){
            //Enter a new anonymous block
            workspace.enter();
            //Map the "data-param" attributes into the workspace...
            auto attrs = node.attributes();
            for(auto attr=attrs.begin();attr!=attrs.end();attr++){
                // Don't try to factor out this string for name. If its not there the 
                // string comparison does not work
                std::string name = attr->name();
                if(name=="data-param"){
                    std::string value = attr->value();
                    //Get the name and value of the parameter
                    //! @todo the parsing of value should be done in a specific method with
                    //! error capture and reporting
                    std::vector<std::string> bits;
                    boost::split(bits,value,boost::is_any_of(":"));
                    std::string parameter = bits[0];
                    std::string expression = bits[1];
                    //Set the parameter in the new block
                    workspace.set(parameter,expression);
                }
            }
        }

        //Render the new children of this node (within the new block)
        render_children(node,workspace);
        
        //Exit the anonymous block if created
        if(params) workspace.exit();
    }
};

}
}
