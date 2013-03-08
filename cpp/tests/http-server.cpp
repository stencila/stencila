#include <iostream>

#include <stencila/http-server.hpp>

int main(void) {
    using namespace Stencila::Http;
    Component<>::declarations();
    try {
        Server server;
        server.run();
    }
    catch (std::exception &e) {
        std::cerr << e.what() << std::endl;
        return 1;
    }
    return 0;
}
