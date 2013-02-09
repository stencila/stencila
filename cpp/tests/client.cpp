#include <iostream>

#include <stencila/http-client.hpp>
#include <stencila/exception.hpp>

int main(void) {
    using namespace Stencila::Http;
    try {
        Client client;
        std::cout<<client.get("qerhghgythe").dump()<<std::endl;
    }
    catch (std::exception &e) {
        std::cerr << e.what() << std::endl;
        return 1;
    }
    return 0;
}
