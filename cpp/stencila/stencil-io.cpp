#include <stencila/stencil.hpp>

namespace Stencila {

Stencil& Stencil::initialise(const std::string& from){
    std::size_t found = from.find("://");
    if(found==std::string::npos){
        // Initialised from a path
        read(from);
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
        if(ext==".html") file<<html(); 
        else if(ext==".cila") file<<cila();
    }
    else STENCILA_THROW(Exception,str(boost::format("File extension <%s> not valid for a Stencil")%ext));
    return *this;
}

Stencil& Stencil::read(const std::string& directory){
    if(directory.length()){
        // Check that directory exits and is a directory
        if(not boost::filesystem::exists(directory)){
            STENCILA_THROW(Exception,str(boost::format("Path <%s> does not exist")%directory));
        }
        if(not boost::filesystem::is_directory(directory)){
            STENCILA_THROW(Exception,str(boost::format("Path <%s> is not a directory")%directory));
        }
        // Set the stencil's path
        path(directory);
    }
    // Currently, set the stencil's content from main.cila
    boost::filesystem::path cila = boost::filesystem::path(directory) / "main.cila";
    if(not boost::filesystem::exists(cila)){
        STENCILA_THROW(Exception,str(boost::format("Directory <%s> does contain a 'main.cila' file")%directory));
    }
    import(cila.string());
    return *this;
}

Stencil& Stencil::write(const std::string& directory){
    // Set `path` if provided
    if(directory.length()) Component::path(directory);
    // Write necessary files
    // @fixme This should use `export_()`
    Component::write("main.html",html(true));
    return *this;
}

}
