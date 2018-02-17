# Functions

Functions are one of the primary plugin mechanisms for Stencila. Stencila provides a seamless interface for using functions written in different programming languages (R, Python, Javascript and more) for anyone, without the need to write the code in the given language. The developers can make their fuctions available to the users via simple API (Application Programming Interface). The users can discover the available functions through the common documentation and apply them to their data.

Stencila function libraries make it easy for users to expand their analysis with an array of methods developed by different research communities.

## Stencila function libraries

Stencila function libraries are collections of functions for data manipulation, analysis and plotting. These collections are grouped per research discipline and programming language. For example, an "R Metagenomics Stencila Library" will contain functions written in R developed and used within the metagenomics research community.

Stencila uses its own simple *glue-like* language [Mini](languages/mini/README.md) which enables the communication between functions written in different programming lanaguages. On its own Mini cannot perform sophisticated calculations but that is not its main purpose. Mini is used for wrapping functions in other languages making them interoperable. Behnid the scenes Stencila uses Mini to call the relevant language context in order to execute the wrapped function.

### Why are Stencila function libraries useful?

Users primarily using spreadsheets for analysing and storing their data have access to a much wider selection of functions than those available in standard spreadsheet applications. By using Stencila functions they can easily keep the record of their workflow aligning their work with the principle of reproducible research.

Developers can make their functions easily available and versatile which results in expanded userbase.

### Stencila Core Library

[Stencila Core Library](https://github.com/stencila/libcore) (LibCore) is a library of functions that are built into Stencila's Mini language. The default implementation of Mini is done in [Javascript](https://github.com/stencila/libcore/tree/master/js). That is, in fact, Mini calls upon Javascript operations. The default implementation in Javascript allows for performing work in Stencila only by the means of a browser. Users who do not have access to other execution contexts (R, Python and so forth) are still able to manipulate their data.

However, CoreLib has also implementation in other languages (for example, [R](https://github.com/stencila/libcore/tree/master/r) and [Python](https://github.com/stencila/libcore/tree/master/py)).

The scope of the Stencila Core Library is corresponding with:
 * a standard set of functions available in most popular spreadsheet applications (for example, in [Excel]((https://support.office.com/en-us/article/Excel-functions-alphabetical-b3944572-255d-4efb-bb96-c6d90033e188));
 * [Python Standard Library](https://docs.python.org/3/library/index.html);
 * [R Base Package](https://stat.ethz.ch/R-manual/R-devel/library/base/html/00Index.html).

See the [list of functions currently implemented](https://stencila.github.io/libcore/#/) in Stencila Core Library..

See detailed guidelines on how to [contriubute to the LibCore](https://github.com/stencila/libcore/blob/master/docs/CONTRIBUTING.md).

### Domain-specific libraries

Domain-specific libraries are collections of functions developed and used within research communities. Using the simple API (by wrapping their functions in a Mini template) anyone can make their code for data analysis, management and plotting available for a wide audience.

Domain-specific libraries are particularily useful for making various tools for data analysis fully interoperable. Researchers who tend to do most of their work in spreadsheets are then able to extend their data analysis by a whole new array of functions in R, Python or other languages.

## Add new functions :sparkles:

Adding functions to Stencila function libraries is fairly straightforward. We provided a [template](https://github.com/stencila/libtemplate) which should help contributors wrap their functions in Mini and write docstrings. As mentioned, Mini is intentionally simple to minimise the effort of adding new functions to the libraries.

We recommend that each library is located in its own separate directory, structured as in this [template](https://github.com/stencila/libtemplate).
 You can either decide to download the template and create the library locally, or you can upload it to GitHub or
 BitBucket (in any case, we recommend keeping it under version control!).

The structure of the new library repository should be simple:

```
docs/
funcs/
    func1.R
    func1.js
    func1.py
    ....
tests/test_func1.R
README.md
LICENSE
```

Your functions should be implemented in the `funcs` directory. There should be just one function per file (i.e. do not implement two functions in the same file).
Follow the templates for each language for writing the documentation strings. When you register a function, the documentation strings will be automatically converted
into documentation files which will be placed in the `docs` folder.

### Register :sparkles:

Once you finished implementing and testing your function, you need to register it to make it available from within Stencila. In order to do
that select `Register function` from the  menu and point to the main directory (for example, `libgenomics`) where the the file with the function is located. Stencila will automatically
 create the documentation from the docstring. You can then use the function within Stencila.

 If you want to make the function available for someone else using Stencila on a different machine, select `Create function package`, then point
 to the man directory with function file. Once the function package is created, select where you want to save it. You can then share the package (which
 essentially is a `zip` file). If the main directory with the function exists as a GitHub or BitBucket repository (see [these guidelines](https://github.com/stencila/libtemplate)),
 then you can simply point users to the repository (which needs to be public).

 To register the function from the package, use the `Register function` option from the menu, as described above. If you are registering from a GitHub or BitBucket repository,
 just copy and paste the link to it.
