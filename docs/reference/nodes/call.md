A simple call of a sibling fixture,

/code-chunk.md()

With arguments,

/code-chunk.md(arg1=val1 arg2=val2 arg3=val3)

Arguments can be symbols defined elsewhere in the document,

/code-chunk.md(arg1=sym1 arg2=some_Symbol_9)

As with `Include`, paths are relative and the `select` option is available,

/../md/paragraph.md(){select=content.1}
