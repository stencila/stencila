#pragma once

#include <string>
#include <vector>

namespace Stencila {
namespace Host {

/**
 * Get an environment variable
 * 
 * @param  name Name of variable
 */
std::string env_var(const std::string& name);

/**
 * Get the path to the user's Stencila store
 */
std::string user_store(void);

/**
 * Get the path to the system wide Stencila store
 */
std::string system_store(void);

/**
 * 'Private' cache of store directories (see `stores()`)
 */
extern std::vector<std::string> stores_;

/**
 * Get the filesystem paths of the Stencila stores
 *
 * `STENCILA_STORES` can be set as an environment variable.
 * It serves the same function as [`PYTHONPATH` in Python](https://docs.python.org/2/using/cmdline.html#envvar-PYTHONPATH) 
 * and [`R_LIBS` in R](http://stat.ethz.ch/R-manual/R-devel/library/base/html/libPaths.html)
 */
std::vector<std::string> stores(void);

/**
 * Get a filesystem path within the primary Stencila store
 * corresponding to the address
 */
std::string store_path(const std::string& address);

/**
 * Generate a temporary directory name
 */
std::string temp_dirname(void);

/**
 * Generate a temporary file name
 *
 * @param extension File name extension for the file
 */
std::string temp_filename(const std::string& extension="");


}
}
