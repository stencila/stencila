%{
  #include "syntax-excel.h"
%}

%option prefix="Excel"
%option reentrant warn
%option noyywrap

%%

"TRUE"|"FALSE"            return BOOLEAN;
((\.[0-9]+)|([0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?))%?     return NUMBER;

\$?[A-Z]+\$?[0-9]+        return IDENTIFIER;
[A-Z]+                    return FUNCTION;

"="                       return EQUAL;
"+"                       return PLUS;
"-"                       return MINUS;
"*"                       return TIMES;
"/"                       return DIVIDE;
"^"                       return CARET;
"<"                       return LT;
">"                       return GT;
"<="                      return LTE;
">="                      return GTE;
"<>"                      return NEQUAL;

"%"                       return PERCENT;
"&"                       return AMPER;
","                       return COMMA;
":"                       return COLON;

"("                       return PAREN_L;
")"                       return PAREN_R;
"["                       return SQUARE_L;
"]"                       return SQUARE_R;
"{"                       return CURLY_L;
"}"                       return CURLY_R;

[ \t\n]+                  ; // Ignore whitespace

.                         return UNRECOGNIZED;

%%
