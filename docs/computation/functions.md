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

## Adding new functions

Adding functions to Stencila function libraries is fairly straightforward. We provided templates for different languages (see below) which should help contributors wrap their functions in Mini and write documentation. As mentioned, Mini is intentionally simple to minimise the effort of adding new functions to the libraries.

In order to add your function to the selected library, please use the template provided for the relevant programming lanague:

* **R functions** [template](languages/r/README.md)

* **Python functions** [template](languages/python/README.md)

* **Javascript functions** [template](languages/js/README.md)

## Creating new libraries
We recommend that each library is located in its own separate directory, structured as in this [template](https://github.com/stencila/libtemplate). Preferably, it should be a public repository.
The structure of the new library repository should be simple:

```
func/
    functionA.R
    functionB.js
    functionB.py
    ....
    tests/
README.md
LICENSE
```
