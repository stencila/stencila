# Operations :sparkles:

> :sparkles: Currently, the operations in Stencila are still under development and
> many of the described here features may work only in the development version. We
> welcome any [help with Stencila development](CONTRIBUTING.md)!

Stencila performs computations on data using _operations_.
Operations are objects encapsulating what should be done to what.


There are

- `set`
- `get`
- `call`
- `function`

Commands are just values. Homoiconic - code is data

## Set


## Get

## Call

## Function

A function is an `object` of type `function` with properties:

- `pars` : an array of the function's parameters
- `body` : a value representing the function body

For example here is a function, which has no parameters and returns the value of _pi_:

```mini
{
  type: 'function',
  body: 3.14159265359
}
```

A function call is an `object` of type `call` with properties:

- `name` : the name of the object being called
- `args` : an array of values to be used as the functions parameters


For example, the simple multiplication of two values,

```mini
3 * 4
```

is equivalent to executing a `call` of function `multiply` like this,

```mini
execute {
  type: 'call',
  func: {type: 'get', name: 'multiply'},
  args: [ 3, 4 ]
}
```

An here's a slightly more complex example of a function which multiplies a number by it's self (i.e. squares it):

```mini
{
  type: 'function',
  pars: [{
    name: 'x',
    type: 'number'
  }],
  body: [{
    type: 'call',
    name: {type: 'get', name: 'multiply'},
    args: [
      {type: 'get', 'x'},
      {type: 'get', 'x'}
    ]
  }]
}
```

## Defining external functions

```mini
{
  type: 'function',
  pars: [{
    name: 'x',
    type: 'number'
  }],
  body: [{
    type: 'external',
    lang: 'r',
    code: 'x * x'
  }]
}
```

External functions can be in Mini

```mini

function (*) r >>>
  x * x
<<<
```
