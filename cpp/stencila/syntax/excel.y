%include {
  #include <cassert>
  #include <iostream>

  #include <stencila/syntax/syntax.hpp>
  using namespace Stencila::Syntax;
}

%extra_argument { Parser* parser }

%token_type { char* }

%type expr { Node* }
%type id { Node* }

%type args { std::vector<Node*>* }
%destructor args { delete $$; }

%left 
  EQUALS
  PAREN_L PAREN_R 
  SQUARE_L SQUARE_R 
  CURLY_L CURLY_R 
  NEWLINE
  FUNCTION
.

%left PLUS MINUS.
%left TIMES DIVIDE.

start ::= expr(e). {
  parser->message = "finished";
  parser->root = e;
}

expr(a) ::= FUNCTION(b) PAREN_L args(c) PAREN_R. {
  a = new Call(b,*c);
}

args(a) ::= args(b) COMMA expr(c). {
  a = b;
  a->push_back(c);
}

args(a) ::= expr(b). {
  a = new std::vector<Node*>(1,b);
}

args(a) ::= . {
  a = new std::vector<Node*>;
}


expr(e) ::= expr(l) PLUS expr(r). {
  e = new Binary('+',l,r);
}

expr(e) ::= expr(l) MINUS expr(r). {
  e = new Binary('-',l,r);
}

expr(e) ::= expr(l) TIMES expr(r). {
  e = new Binary('*',l,r);
}

expr(e) ::= expr(l) DIVIDE expr(r). {
  e = new Binary('/',l,r);
}

expr(e) ::= BOOLEAN(v). {
  e = new Boolean(v);
}

expr(e) ::= NUMBER(v). {
  e = new Number(v);
}

expr(e) ::= STRING(v). {
  e = new String(v);
}

expr(a) ::= id(b) COLON id(c). {
  a = new Range(b,c);
}

id(a) ::= IDENTIFIER(id). {
  a = new Identifier(id);
}

expr ::= UNRECOGNIZED(b). {
  STENCILA_THROW(Stencila::Exception, std::string("Unrecognised character: ") + b);
}


%syntax_error {  
  STENCILA_THROW(Stencila::Exception, "Excel parser syntax error");
}

%parse_failure {
  STENCILA_THROW(Stencila::Exception, "Excel parser failure");
}

%stack_overflow {
  STENCILA_THROW(Stencila::Exception, "Excel parser stack overflow");
}
