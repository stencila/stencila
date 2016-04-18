#include <stencila/syntax/excel.hpp>

#include "excel-lexer.hpp"
#include "excel-parser.hpp"

void* ParseAlloc(void* (*allocProc)(size_t));
void Parse(void* lemon, int, char*, Parser* parser);
void ParseFree(void* lemon, void(*freeProc)(void*));

namespace Stencila {
namespace Syntax {

ExcelParser::ExcelParser(void) {
    init(ParseAllo, Parse, ParseFree);
}

}
}
