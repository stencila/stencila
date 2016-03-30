#include <cstdlib>
#include <iostream>
#include <string>
#include <vector>

#include "lexer.hpp"
#include "parser.hpp"

#include <stencila/syntax/syntax.hpp>
using namespace Stencila::Syntax;

#include <stencila/syntax/r.hpp>
 
void* ParseAlloc(void* (*allocProc)(size_t));
void Parse(void* lemon, int, char*, Parser* parser);
void ParseFree(void* lemon, void(*freeProc)(void*));

namespace Stencila {
namespace Syntax {

void Parser::parse(const std::string& line) {
    // Create the Flex lexer and get it to
    // scan the line
    yyscan_t lexer;
    yylex_init(&lexer);
    YY_BUFFER_STATE buffer_state = yy_scan_string(line.c_str(), lexer);

    // Create the Lemon parser
    void* lemon = ParseAlloc(malloc);

    // Due to an interaction between the memory management
    // of Flex and Lemon it is neccessary to do `strdup(yyget_text(lexer))`
    // below and then free this memory later
    // See http://stackoverflow.com/a/20713882/4625911
    std::vector<char*> texts;

    // Iterate over lexer tokens
    while (int code = yylex(lexer)) {
        char* text = strdup(yyget_text(lexer));
        texts.push_back(text);
        Parse(lemon, code, text, this);
    }
    Parse(lemon, 0, NULL, this);

    // Clean up the lexer
    yy_delete_buffer(buffer_state, lexer);
    yylex_destroy(lexer);

    // Cleanup the lemon
    ParseFree(lemon, free);

    // Cleanup the duplicated strings
    for (auto text : texts) free(text);
}

void Parser::show(void) const {
    std::cout<<message<<std::endl;
    if (root) {
        TreeGenerator(std::cout).visit(root);
        std::cout << std::endl << std::endl;
        RGenerator(std::cout, "excel").visit(root);
    }
}

}
}

int main() {
    std::string line;
    std::getline(std::cin, line);
    Parser parser;
    parser.parse(line);
    parser.show();
    return 0;
}