#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

Stencil& Stencil::initialise(const std::string& from){
    std::size_t found = from.find("://");
    if(found==std::string::npos){
        // Initialised from an address or path
        if(boost::filesystem::exists(from)){
            // This is a path so read from it
            read(from);
        } else {
            // Search for address
            std::string path = Component::locate(from);
            if(path.length()) read(path);
            else STENCILA_THROW(Exception,"No stencil found with path or address <"+from+">");
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
    if(not boost::filesystem::exists(path)){
        STENCILA_THROW(Exception,"File <"+path+"> not found");
    }
    std::string ext = boost::filesystem::extension(path);
    if(ext==".html" or ext==".cila"){
        std::ifstream file(path);
        std::stringstream stream;
        stream<<file.rdbuf();
        std::string content = stream.str();
        if(ext==".html") html(content); 
        else if(ext==".cila") cila(content);
    }
    else STENCILA_THROW(Exception,"File extension <"+ext+"> not valid for a Stencil");
    return *this;
}

Stencil& Stencil::export_(const std::string& path){
    std::string ext = boost::filesystem::extension(path);
    if(ext==".html" or ext==".cila"){
        std::ofstream file(path);
        if(ext==".html") file<<html(true,true); 
        else if(ext==".cila") file<<cila();
    }
    else STENCILA_THROW(Exception,"File extension <"+ext+"> not valid for a Stencil");
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
            STENCILA_THROW(Exception,"Path <"+where+"> does not exist");
        }
        if(not boost::filesystem::is_directory(where)){
            STENCILA_THROW(Exception,"Path <"+where+">> is not a directory");
        }
        // Set the stencil's path
        path(where);
    }
    // Search for a stencil.html file and, if it exists, read it using `import`
    boost::filesystem::path filename = boost::filesystem::path(path()) / "stencil.html";
    if(boost::filesystem::exists(filename)) import(filename.string());
    
    return *this;
}

Stencil& Stencil::write(const std::string& path_arg){
    if(path_arg.length()) path(path_arg);
    Component::write("stencil.html",html(true));
    return *this;
}

}
