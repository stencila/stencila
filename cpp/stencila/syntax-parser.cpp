#include <cstdlib>
#include <string>
#include <vector>

#include <stencila/syntax/parser.hpp>

namespace Stencila {
namespace Syntax {

void Parser::init(ParseAlloc parse_alloc, Parse parse, ParseFree parse_free) {
    parse_alloc_ = parse_alloc;
    parse_ = parse;
    parse_free_ = parse_free;
}

void Parser::parse(const std::string& string) {
    // Create the Flex lexer and get it to
    // scan the string
    yyscan_t lexer;
    yylex_init(&lexer);
    YY_BUFFER_STATE buffer_state = yy_scan_string(string.c_str(), lexer);

    // Create the Lemon parser
    void* lemon = parse_alloc_(malloc);

    // Due to an interaction between the memory management
    // of Flex and Lemon it is neccessary to do `strdup(yyget_text(lexer))`
    // below and then free this memory later
    // See http://stackoverflow.com/a/20713882/4625911
    std::vector<char*> texts;

    // Iterate over lexer tokens
    while (int code = yylex(lexer)) {
        char* text = strdup(yyget_text(lexer));
        texts.push_back(text);
        parse_(lemon, code, text, this);
    }
    parse_(lemon, 0, NULL, this);

    // Clean up the lexer
    yy_delete_buffer(buffer_state, lexer);
    yylex_destroy(lexer);

    // Cleanup the lemon
    parse_free_(lemon, free);

    // Cleanup the duplicated strings
    for (auto text : texts) free(text);
}

const Node* Parser::tree(void) const {
    return root_;
}

}
}
