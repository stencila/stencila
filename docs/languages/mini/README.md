# Mini

## Data

The simplest atomic type is `boolean` which can only have one of two values:

```mini
true
```

or

```mini
false
```

```mini
3.141592654
```

```mini
42
```

Strings are sequences of characters. String literals can use either single or double quotation marks.

```mini
'hello world'
```

```mini
"hello world"
```

An array is a sequence of values e.g. 

```mini
[1, 2, 3]
```

The values in an array can have different types e.g.

```mini
['hello world', [1, 2], {}]
```

An object is a collection of values in which eack value has a _key_ e.g.

```mini
{a: 1, b: 2}
```

## Calls

Functions are called using parentheses containing arguments: e.g

```mini
add(1, 2)
```

Named arguments can be used, but only after unnamed arguments. e.g.

```mini
add(1, other=2)
```

## Operators

Most operators in Mini are simply shortcuts to writing function calls. For example, the forward slash operator `/` is a short cut for the `divide` function, so the expression `4/5`, is equivalent to the function call expression `divide(4, 5)`. 

This allows you to write shorter, more readable and comprehensible expressions. For example, instead of writing a nested set of calls like:

```mini
and(less(add(a, b), 10), equal(c, 1))
```

You can write:

```mini
a + b < 10 && c == 4
```

However, there are two operators, the dot (`.`) and the (`|`) pipe which behave differently.

### Dot operator

The dot operator, `.`, is used to select members from an object or table. For example, to get a column `age` from a table named `data` you use `data.age` which is equivalent to `select(data, 'age')`. When used in this way, the dot operator acts like a function call shortcut just like the other operators.

But the dot operator can also be used to define a _symbol_ within a syntax expression. A syntax expression is a partially evaluated expression that can be used as an argument in a function call so that it's evaluation can be completed within an alternative scope. For example, the second parameter of the `filter` function is a syntax expression e.g. 

```mini
filter(data, .age < 40)
```

For more see [lambdas](lambdas).

### Pipe operator

The pipe operator, `|`, allows for several function calls to be combined in a "pipeline". It passes the expression on the left as the first argument of the function call on the right. So a set of nested function calls like:

```mini
sum(select(filter(data, 'row.age <= 40'), 'weight'))
```

can be written in a more readable pipeline as:

```mini
data | filter('row.age <= 40') | select('weight') | sum()
```

### Operator precedence

Operators have differing levels of precedence. Precedence determines the order in which operators are parsed with respect to each other. Operators with higher precedence become the operands of operators with lower precedence. Operator precedence in Mini is very similar to that in other languages.

The following table list the operators in Mini in order of decreasing precedence (in groups of equal precedence) along with their function call equivalents. See [`stencila/libcore`](https://github.com/stencila/libcore) for implementation and documentation for these functions.

Precedence | Operator | Usage example     | Function call equivalent
:--------- | :------: | :-----------      | :---------------------- 
1          | `.`      | `value.member`    | `select(value, member)`
1          | `[]`     | `value[member]`   | `select(value, member)`
2          | `!`      | `!value`          | `not(value)`
2          | `+`      | `+value`          | `positive(value)`
2          | `-`      | `-value`          | `negative(value)`
3          | `^`      | `value ^ expon`   | `pow(value, expon)`
4          | `*`      | `value * other`   | `multiply(value, other)`
4          | `/`      | `value / other`   | `divide(value, other)`
4          | `%`      | `value % other`   | `remainder(value, other)`
5          | `+`      | `value + other`   | `add(value, other)`
5          | `-`      | `value - other`   | `subtract(value, other)`
6          | `<`      | `value < other`   | `less(value, other)`
6          | `<=`     | `value <= other`  | `less_or_equal(value, other)`
6          | `>`      | `value > other`   | `greater(value, other)`
6          | `>=`     | `value >= other`  | `greater_or_equal(value, other)`
7          | `==`     | `value == other`  | `equal(value, other)`
7          | `!=`     | `value != other`  | `not_equal(value, other)`
8          | `&&`     | `value && other`  | `and(value, other)`
9          | &#124;&#124; | `value `&#124;&#124;` other`  | `or(value, other)`
10         | &#124; | `value `&#124;` cos()`  | `cos(value)`


## Functions

A more convenient way to define a function in Mini is using the `function` keyword. So, our simple function for _pi_ above could be written in Mini using: 

```mini
function() 3.14159265359
```

Because functions are just object values, they can be assigned to variables e.g.

```mini
pi = function() 3.14159265359
```

and then called later

```mini
pi()
```
```mini
3.14159265359
```

> :sparkles: Currently, calling a function like this won't work. That's because Mini expects all functions to be defined externally (see below). Contexts have a `callFunction()` method which takes the name of pre-registed external function. This method, or another similar one, needs to be able to accept a `function` object and call it.

> :sparkles: Currently, only single expression functions can be defined in Mini. It is likely that multi-expression function bodies will be possible in the future.


### Parameters

```mini
function(x, y) x * y
```

```mini
function(x, y = 1) x * y
```

Repeatable parameters (a.k.a variadic parameters)

```mini
function(x, y...) x * sum(y)
```

```mini
show = function(args...) names(args) 
```

### Recursion

Recursive function calls can be useful.... Call themselves. But in Mini, functions do not have access to the global scope - they can only access local variables.

```mini
factorial = function(n) if(n == 0, 1, n * @(n - 1))
```

Compare this to how you would define the same function in other popular languages (using as similar syntax as possible):

```js
const factorial = (n) => n == 0 ? 1 : n * factorial(n - 1)
```
```python
factorial = lambda(n): 1 if n == 0 else n * factorial(n - 1)
```
```r
factorial = function(n) if(n == 0) 1 else n * factorial(n - 1)
```

> :sparkles: Recursive function calls are currently just an idea an are not yet implemented.
