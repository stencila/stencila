#include <stencila/network.hpp>

namespace Stencila {

Server* Server::instance_ = 0;
std::thread* Server::thread_ = 0;

} // namespace Stencila
