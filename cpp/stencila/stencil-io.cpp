#include <stencila/stencil.hpp>

namespace Stencila {

Stencil& Stencil::initialise(const std::string& from){
    std::size_t found = from.find("://");
    if(found==std::string::npos){
        // Initialised from an address or path
        if(boost::filesystem::exists(from)){
            read(from);
        } else {
            std::string path = Component::locate(from);
            if(path.length()) read(path);
            else STENCILA_THROW(Exception,str(boost::format("No stencil found with path or address <%s>")%from));
        }        
    } else {
        // Initialised from some content
        std::string type = from.substr(0,found);
        std::string content = from.substr(found+3);
        if(type=="html") html(content);
        else if(type=="cila") cila(content);
        else if(type=="file") import(content);
        else STENCILA_THROW(Exception,"Unrecognised content type: " + type);
    }
    return *this;
}

Stencil& Stencil::import(const std::string& path){
    std::string ext = boost::filesystem::extension(path);
    if(ext==".html" or ext==".cila"){
        std::ifstream file(path);
        std::stringstream stream;
        stream<<file.rdbuf();
        std::string content = stream.str();
        if(ext==".html") html(content); 
        else if(ext==".cila") cila(content);
    }
    else STENCILA_THROW(Exception,str(boost::format("File extension <%s> not valid for a Stencil")%ext));
    return *this;
}

Stencil& Stencil::export_(const std::string& path){
    std::string ext = boost::filesystem::extension(path);
    if(ext==".html" or ext==".cila"){
        std::ofstream file(path);
        if(ext==".html") file<<html(true,true); 
        else if(ext==".cila") file<<cila();
    }
    else STENCILA_THROW(Exception,str(boost::format("File extension <%s> not valid for a Stencil")%ext));
    return *this;
}

Stencil& Stencil::read(const std::string& directory){
    std::string where = directory;
    if(where.length()==0){
        where = path();
        if(where.length()==0){
            STENCILA_THROW(Exception,"Path not supplied and not yet set for stencil");
        }
    }
    else {
        // Check that directory exits and is a directory
        if(not boost::filesystem::exists(where)){
            STENCILA_THROW(Exception,str(boost::format("Path <%s> does not exist")%where));
        }
        if(not boost::filesystem::is_directory(where)){
            STENCILA_THROW(Exception,str(boost::format("Path <%s> is not a directory")%where));
        }
        // Set the stencil's path
        path(where);
    }
    // Search for a stencil file. Currently with precedence on .cila before .html files
    bool found = false;
    for(std::string extension : {"cila","html"}){
        boost::filesystem::path filename = boost::filesystem::path(where) / ("stencil." + extension);
        if(boost::filesystem::exists(filename)){
            found = true;
            import(filename.string());
            break;
        }
    }
    if(not found) STENCILA_THROW(Exception,str(boost::format("Directory <%s> does contain a 'stencil.html' or 'stencil.cila' file")%where));
    return *this;
}

Stencil& Stencil::write(const std::string& path_arg){
    if(path_arg.length()) path(path_arg);
    // Write necessary files
    // @fixme This should use `export_()`
    Component::write("stencil.html",html(true));
    return *this;
}

}
