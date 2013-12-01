//! @file system.hpp
//! @brief Operating system utilities
//! @author Nokome Bentley

#include <boost/filesystem.hpp>

#pragma once

//! Get the path to the user's Stencila directory which holds Stencila data.
//!
//! This is a first attempt at generating a cross platform home directory path. Note that on Windows
//! and Mac, aplication data usually goes in specific directories, not the ".stencila" directory as is *nix convention
//! See:
//!     http://stackoverflow.com/questions/4891006/how-to-create-a-folder-in-the-home-directory
//!     http://stackoverflow.com/questions/2552416/how-can-i-find-the-users-home-dir-in-a-cross-platform-manner-using-c
//!     http://stackoverflow.com/questions/2910377/get-home-directory-in-linux-c
//! @return Path to the user's Stencila directory
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
