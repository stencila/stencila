#pragma once

namespace Stencila {
namespace Helpers {

// Create a script in temporary directory if it does not yet exist
// This is done to avoid having permanent scripts in a folder that may 
// vary by package (e.g. R, Python) and OS
std::string script(const std::string& filename,const std::string& contents);

// Execute a system command
void execute(const std::string& command);

// Call a system command and return output
std::string call(const std::string& command);

}
}
