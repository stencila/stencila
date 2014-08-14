#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::html(bool document,bool indent) const {
    if(not document){
        // Return content only
        return Xml::Document::dump(indent);
    } else {
        // Return a complete HTML document
        Html::Document doc;
        typedef Xml::Node Node;

        // Being a valid HTML5 document, doc already has a <head> <title> and <body>
        Node head = doc.find("head");
        Node body = doc.find("body");

        // Title to <title>
        // Although we are creating an XHTML5 document, if the title tag is empty (i.e <title />)
        // this can cause browser parsing errors. So always ensure that there is some title content.
        auto t = title();
        if(t.length()==0) t = "Untitled";
        head.find("title").text(t);

        // Keywords to <meta>
        auto k = keywords();
        if(k.size()>0){
            head.append("meta",{
                {"name","keywords"},
                {"content",boost::algorithm::join(k,", ")}
            });
        }

        // Description to <meta>
        auto d = description();
        if(d.length()>0){
            head.append("meta",{
                {"name","description"},
                {"content",d}
            });
        }

        // The following tags are added with a space as content so that they do not
        // get rendered as empty tags (e.g. <script... />). Whilst empty tags should be OK with XHTML
        // they can cause problems with some browsers.

        /**
         * <link rel="stylesheet" ...
         *
         * Links to CSS stylesheets are [placed in the head](http://developer.yahoo.com/performance/rules.html#css_top) 
         * Use a site-root relative path (by adding the leading forward slash) so that CSS can be served from 
         * localhost.
         */
        std::string css = "/" + theme() + "/theme.css";
        head.append("link",{
            {"rel","stylesheet"},
            {"type","text/css"},
            {"href",css}
        }," ");

        /**
         * #contexts
         *
         * Added as a <ul> in body
         */
        auto c = contexts();
        if(c.size()>0){
            Node contexts_elem = body.append("ul",{
                {"id","contexts"}
            }," ");
            for(auto context : c){
                contexts_elem.append("li",{
                    {"class",context}
                },context);
            }
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
        auto a = authors();
        if(a.size()>0){
            auto authors_elem = body.append("address",{
                {"id","authors"}
            }," ");
            for(auto author : authors()){
                authors_elem.append("a",{
                    {"rel","author"},
                    {"href","#"}
                },author);
            }
        }

        /**
         * #content
         *
         * Content is placed in a <main> rather than just using the <body> so that extra HTML elements can be added by the 
         * theme without affecting the stencil's content.
         */
        auto content = body.append("main",{
            {"id","content"}
        }," ");
        content.append(*this);

        /**
         * <script>
         *
         * Script elements are [placed at bottom of page](http://developer.yahoo.com/performance/rules.html#js_bottom)
         * Files are with a fallback to hub.
         */
        body.append("script",{{"src","/core/themes/boot.js"}}," ");
        body.append("script","if(!window.Stencila){window.StencilaHost='http://hub.stenci.la';document.write(unescape('%3Cscript src=\"http://hub.stenci.la/core/themes/boot.js\"%3E%3C/script%3E'))}");
        body.append("script","window.Stencila.Booter.theme('" + theme() + "');");

        return doc.dump(indent);
    }
}


Stencil& Stencil::html(const std::string& html){
    Html::Document doc(html);
    typedef Html::Node Node;

    // Being a valid HTML5 document, doc already has a <head> <title> and <body>
    // so these do not have to be checked for
    Node head = doc.find("head");
    Node body = doc.find("body");

    // Title
    title(head.find("title").text());

    // Keywords
    if(Node elem = head.find("meta","name","keywords")){
        std::string content = elem.attr("content");
        std::vector<std::string> items;
        boost::split(items,content,boost::is_any_of(","));
        for(auto& keyword : items) boost::trim(keyword);
        keywords(items);
    }

    // Description
    if(Node elem = head.find("meta","name","description")){
        std::string content = elem.attr("content");
        description(content);
    }       

    // Contexts
    if(Node elem = body.find("ul","id","contexts")){
        std::vector<std::string> items;
        for(auto& item : elem.all("li")){
            std::string context = item.text();
            boost::trim(context);
            if(context.length()) items.push_back(context);
        }
        contexts(items);  
    }

    // Authors
    if(Node elem = body.find("address","id","authors")){
        std::vector<std::string> items;
        for(auto& item : elem.all("a[rel=\"author\"]")){
            items.push_back(item.text());
        }
        authors(items);  
    }

    // Content
    // Clear content before appending new content from Html::Document
    clear();
    if(Node elem = body.find("main","id","content")){
        append_children(elem);
    }
    append_children(doc.find("body"));
    return *this;
}

} //namespace Stencila
