#pragma once

#include <boost/filesystem.hpp>

#include <stencila/exception.hpp>

namespace Stencila {

static std::vector<std::string> libraries_;

static std::vector<std::string>& libraries(void){
    return libraries_;
}

/**
 * Get the path to the user's Stencila directory 
 * 
 * This function attempts to generat3 a cross platform home directory path. Note that on Windows
 * and Mac, aplication data usually goes in specific directories, not the ".stencila" directory as is *nix convention
 * See:
 *     http://stackoverflow.com/questions/4891006/how-to-create-a-folder-in-the-home-directory
 *     http://stackoverflow.com/questions/2552416/how-can-i-find-the-users-home-dir-in-a-cross-platform-manner-using-c
 *     http://stackoverflow.com/questions/2910377/get-home-directory-in-linux-c
 */
static std::string home(void) {
    std::string home = std::getenv("HOME");
    if(not home.length()) {
        home = std::getenv("USERPROFILE");
    }
    if(not home.length()) {
        std::string home_drive = std::getenv("HOMEDRIVE");
        std::string home_path = std::getenv("HOMEPATH");
        home = home_drive+home_path;
    }
    if(not home.length()) {
        home = boost::filesystem::current_path().string();
    }
    return home + "/.stencila/";
}

static std::string library_user(void) {
    std::string path = home() + "library/";
    boost::filesystem::create_directories(path);
    return path;
}

static std::string library_system(void) {
    std::string path = "/usr/lib";
    return path;
}

static void initialise(void){
    libraries_ = {
        "file://.",
        library_user(),
        library_system(),
        "http://stenci.la"
    };
}

/**
 * Locate the component having the `address`
 * 
 * @param  address [description]
 * @return         URL of the component (`http://` or `file://`)
 */
static std::string locate(const std::string& address){
    std::string url = "";
    throw Exception("Component with address not found: "+address);
    return url;
}

/**
 * Obtain the component having the `address` and optionally meeting a `version` requirement
 *
 * 
 * 
 * @param  address     [description]
 * @param  version     [description]
 * @param  comparision [description]
 * 
 * @return             [description]
 */
template<class Class>
static Class obtain(const std::string& address,const std::string& version="",const std::string& comparision="=="){
    // First, check in the cache for a component with that address
    
    // Not found in cache so locate the component and read from that URL
    std::string url = locate(address);
    Class comp(url);
    return comp;
}

}
