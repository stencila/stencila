#include <iostream>

#include <stencila/server.hpp>

int main(void) {
    using namespace Stencila;
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
