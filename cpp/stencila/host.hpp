#pragma once

namespace Stencila {
namespace Host {

std::string current_dir(void);

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
std::string user_dir(void);

std::string system_dir(void);

std::string temp_dirname(void);

std::string temp_filename(const std::string& extension="");

}
}
