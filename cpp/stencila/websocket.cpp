#include <stencila/websocket.hpp>

namespace Stencila {
namespace Websocket {

Server* Server::instance_ = 0;
std::thread* Server::thread_ = 0;

} // namespace Websocket
} // namespace Stencila
