# Engine

The `Engine` is resposible for deciding _when_ (i.e. in which order) and _where_ (i.e. in which execution `Context`) cells are executed.
It can be thought of as a meta-`Context`, that orchestrates other `Contexts`.

The `Engine` takes the code from a cell and converts it into an operation which it then passes
to the `execute` method of the `Context` which executes the operation and returns.

## Cells

With Stencila you have full control over the sequence in which your code cells are executed. You can run the code in asynchronous order.
You can refer to specific outputs and inputs from the given cell in any part of your Stencila document. Depending
on the programming language which you use in the cell, you may need to capture the output explicitly, in order to be able to
refer to it. For more details see the documentation for using different programming languages in Stencila.

If you do not capture the output explicitly, you will not be able to refer to it later on. But the input of the cell
will still be available.


