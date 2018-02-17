# Python

Stencila allows you use interactive Python code accross the whole Stencila suite (atricles, notebooks and sheets).
In order to be able to use Python within Stencila documents you need to have the
[Python execution context](getting-started/installation.md#execution-contexts) enabled. You can write Python code
just like you would in any other editor or reproducible notebook. You can install Python packages and import them,
create and embed plots, and so on.

## Data interchange
Stencila provides you with ability to use multiple programming languages to write interactive code within
one document, working on the same data. You can manipulate the same data switching between different programming
languages. This capability is achieved through `data interchange` feature.

When you pass data between cells Stencila temporarily converts it into its built-in [Mini language](languages/mini/README.md) object.
The table below shows (roughly) how data interchange between Mini and Python is implemented. For more details
see [source code](https://github.com/stencila/py/blob/master/stencila/value.py).

| Mni     | Python           |
| ------- | ---------------- |
| boolean | bool             |
| integer | int              |
| float   | float            |
| string  | str              |
| table   | pandas.DataFrame |
| object  | Python object*   |
| array   | Python object*   |


*The object fields and methods are saved within the Mini object/array and converted accordingly.


## Cells
With Stencila you have full control over the sequence in which your code cells are executed. You can run the code in asynchronous order.
You can refer to specific outputs from the given cell in any part of your Stencila document.
Stencila does all this using its [execution engine](computation/engine.md).

The engine manages automatic dependencies between the cells which are specific for each language.


## Functions
You can use Python functions (either from Python packages or the ones you wrote yourself) in all Stencila documents.
In Stencila notebooks and articles you just write Python code as normally. In Stencila Sheets you can call Python functions
using the following syntax in the cells: `py=  .....`. For example `py= import numpy as np` :sparkles:

You can also make your Python functions available through the menu in Stencila Spreadsheet :sparkles: . This means that a user
who interacts with the data primarily using the Stencila Spreadsheet will be able to apply your Python functions to the
data without having to know how to code.

In order to register your Python function with Stencila, you need to first [create a simple package](computation/functions.md#adding-new-functions)
using Stencila API.

Each function should be included in a separate file with the `.py` extension (so, for example, `my_function1.py`) :sparkles: .
The specification for your Python function should be written in a [Python docstring](https://www.python.org/dev/peps/pep-0257/) following
the [Stencila libtemplate](https://github.com/stencila/libtemplate).

### Test

All functions should have tests written

### Register :sparkles:
Once you finished implementing and testing your Python function, you need to register it to make it available from within Stencila. In order to do
that select `Register function` from the  menu and point to the main directory (for example, `libgenomics`) where the `.py` file with the function is located. Stencila will automatically
 create the documentation from the docstring. You can then use the function within Stencila.

 If you want to make the function available for someone else using Stencila on a different machine, select `Create function package`, then point
 to the man directory with function file. Once the function package is created, select where you want to save it. You can then share the package (which
 essentially is a `zip` file). If the main directory with the function exists as a GitHub or BitBucket repository (see [these guidelines](https://github.com/stencila/libtemplate)),
 then you can simply point users to the repository.

 To register the function from the package, use the `Register function` option from the menu, as described above. If you are registering from a GitHub or BitBucket repository,
 just copy and paste the link to it.
