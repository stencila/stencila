#include <iostream>

#include <stencila/stencil.hpp>

int main(void){
    using namespace Stencila;

    Stencil s;
    s.title("A stencil");
    s.html("Hellowww!!!");
    s.view();

    std::cout<<"Press enter to exit\n";
    std::cin.ignore();
}
