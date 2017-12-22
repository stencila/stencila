# Introduction

The Engine is doing the work of evaluating expressions and computing the result. It is important that the Engine does not block the application when executed. This means, the UI should not freeze or have a slower response on user interactions while the Engine evaluates cells. For that the Engine needs to be implemented in a different way.

# Proposal

Run the Engine in an eytra thread using a (web-)worker.
Cells and their values live primilary in the Engine's thread. Only for rendering and for cell state updates, messages are sent to the view.

In the heart of the engine, a scheduler is run, propagating cell state updates and computing results. The variables used in an expression form a dependency graph, that is considered by the scheduler to achieve a correct order of execution.

If a cell is changed, all other depending cells are updated, too.
If a cell is created or removed, the dependency graph is updated accordingly.

A cell has a life-cycle: initially it is just a raw piece of source code, then it is analysed to forming an expression potentially depending on other cells or exposing a variable, then it is a evaluated having a result value or an error, and finally it is validated.

```
sum(1 2)
```
is a syntactical error.

```
sum(x, abc)
```
could be dependency error if 'abc' was not a defined variable.

```
sum('foo')
```
would lead to a runtime error because `sum()` can not be applied to strings.

```
-1
```
Could be invalid in a cell that allows only positive values.
