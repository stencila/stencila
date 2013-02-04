#include <iostream>

#include <stencila/rest-server.hpp>
#include <stencila/exception.hpp>

int main(void) {
    using namespace Stencila::Rest;
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
