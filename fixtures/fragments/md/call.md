A simple call of a sibling fixture,

/code-chunk.md()

With arguments,

/code-chunk.md(arg1=1 arg2="2" arg3=3.14)

Arguments can be symbols defined elsewhere in the document,

/code-chunk.md(arg1=sym1 arg2=some_Symbol_9)

As with `Include`, paths are relative and the `select` option is available,

/../md/paragraph.md(){select=content.1}
