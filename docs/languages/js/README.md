# Javascript

Cells and functions can be written using Javascript. An execution context for Javascript, `JavascriptContext` is implemented in the [`stencila/js`](https://github.com/stencila/js) repository so it's already availble to all Stencila documents without the need for external hosts.

## Data interchange



## Cells



## Functions


Stencila functions can be implemented using Javascript. To match Stencila's call semantics with Javascript's it is necessary to set the `pars` property on the Javascript function


```js
function square(value) {
  return value * value
}
square.pars = ['value']
```


When a parameter is repeatable, prefix the argument name with an ellipsis e.g.

```js
function sum(...value) {
  // Implementation goes here
}
sum.pars = ['...value']
```

```mini
sum(3, [])
```

```js
function sum(...values) {
  let result = 0
  for (let value of values) {
    const type_ = type(value)
    switch (type_) {
      case 'number':
        result += value
      case 'array[number]':
        for (let item of value) result += value
      default:
        type_error(type_)
    }
  }
  return result
}
sum.pars = ['...values']
```
