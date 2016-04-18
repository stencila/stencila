/**
 * A template C++ file for the `parse()` method of Parser
 * classes used to link to the correct Flex and Lemon generated
 * functions
 */
#include <stencila/syntax-{lang}.hpp>
#include "syntax-{lang}-lexer.hpp"

void* {lang-title}Alloc(void* (*)(size_t));
void  {lang-title}(void*, int, char*, Stencila::Syntax::Parser*);
void  {lang-title}Free(void*, void(*)(void*));

namespace Stencila {
namespace Syntax {

const Node* {lang-title}Parser::parse(const std::string& string) {
    // Create the Flex lexer and get it to
    // scan the string
    void* lexer;
    {lang-title}lex_init(&lexer);
    {lang-title}_scan_string(string.c_str(), lexer);

    // Create the Lemon parser
    void* lemon = {lang-title}Alloc(malloc);

    // Due to an interaction between the memory management
    // of Flex and Lemon it is neccessary to do `strdup({lang-title}get_text(lexer))`
    // below and then free this memory later
    // See http://stackoverflow.com/a/20713882/4625911
    std::vector<char*> texts;

    // Iterate over lexer tokens
    while (int code = {lang-title}lex(lexer)) {
        char* text = strdup({lang-title}get_text(lexer));
        texts.push_back(text);
        {lang-title}(lemon, code, text, this);
    }
    {lang-title}(lemon, 0, NULL, this);

    // Clean up the lexer
    {lang-title}lex_destroy(lexer);

    // Cleanup the lemon
    {lang-title}Free(lemon, free);

    // Cleanup the duplicated strings
    for (auto text : texts) free(text);

    return root_;
}

}
}
