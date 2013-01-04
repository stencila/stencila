/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file registry.hpp
//! @brief Definition of Registry which holds information on local stencila objects
//! @author Nokome Bentley

#pragma once

#include <stencila/dataset.hpp>

namespace Stencila {

class Registry : public Dataset {
public:
    Registry(void):
        Dataset(home()+"stencila.sds"){
        execute(
            "CREATE TABLE IF NOT EXISTS \"stencils\" (id TEXT, content TEXT);"
        );
    }

    /*!
    Return the path to the users Stencila directory which holds application data.
    This is a first attempt at a cross platform path but note that on Windows
    and Mac aplication data usually goes in specific directories, not the ".stencila" directory
    as is *nix convention
    See:
        http://stackoverflow.com/questions/4891006/how-to-create-a-folder-in-the-home-directory
        http://stackoverflow.com/questions/2552416/how-can-i-find-the-users-home-dir-in-a-cross-platform-manner-using-c
        http://stackoverflow.com/questions/2910377/get-home-directory-in-linux-c
    */
    
    //! @brief 
    //! @return 
    std::string home(void) const {
        std::string home = std::getenv("HOME");
        if(not home.length()) {
            home = std::getenv("USERPROFILE");
        }
        //! @brief 
        //! @param length
        //! @return 
        if(not home.length()) {
            std::string home_drive = std::getenv("HOMEDRIVE");
            std::string home_path = std::getenv("HOMEPATH");
            home = home_drive+home_path;
        }
        //! @brief 
        //! @param length
        //! @return 
        if(not home.length()) {
            home = boost::filesystem::current_path().string();
        }
        return home + "/.stencila/";
    }

} Registry;

}
