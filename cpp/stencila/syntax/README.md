For full import/export from/to other spreadsheet formats (e.g. Excel `.xlsx`, Google Sheets) it is necessary to parse/generate formulas in cells. For example, if importing an Excel spreadsheet into a Stencila R Sheet it would be necessary to translate `AVERAGE(A1:A10)` to `mean(A1:A10)`, and do the reverse if exporting. Some proposals for how to do this in a way that is extendible to multiple source/target languages:

Implement an [Abstract Syntax Tree (AST)](https://en.wikipedia.org/wiki/Abstract_syntax_tree) in simple C++ classes. Only expression nodes need to be implemented e.g. expression, call, operators. The AST would be internal representation used for translation e.g. `Excel -> ExcelParser -> AST -> RGenerator -> R`.  AST nodes would only contain data and all the translation logic would be in Parser and Generator classes.

Implement languages parsers using [Flex](https://en.wikipedia.org/wiki/Flex_(lexical_analyser_generator)) + [Lemon](https://en.wikipedia.org/wiki/Lemon_Parser_Generator) to generate an AST from a language expression. For the 


"OpenFormula is an open standard for exchanging recalculated formulae in spreadsheets"
https://en.wikipedia.org/wiki/OpenFormula
http://www.dwheeler.com/openformula/

http://www.dwheeler.com/openformula/of.l
http://www.dwheeler.com/openformula/of.y


[Official(?) documentation](http://www.hwaci.com/sw/lemon/lemon.html)


[Mike Chirico's Lemon tutorial circa 2004](http://souptonuts.sourceforge.net/readme_lemon_tutorial.html)
[Notes on converting a Bison to Lemon](http://brlcad.org/websvn/filedetails.php?repname=BRL-CAD&path=%2Fbrlcad%2Ftrunk%2Fdoc%2Fbison_to_lemon.txt&usemime=1&rev=51813)

[Troy Hanson's notes](https://troydhanson.github.io/lemon_notes.html)

https://github.com/troydhanson/misc/blob/master/lemon/lemon.txt

https://brskari.wordpress.com/2012/04/30/writing-a-simple-shell-using-flex-and-lemon-part-2/