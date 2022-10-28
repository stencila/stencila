An example mainly intended for testing `If` nodes of different shapes and sizes.

# No expression, language, or content

Mainly a test that Markdown parsing, and Web Components, work when there is no expression, language or content

::: if

::: elif

::: else

:::

# No language specified

Language will be guessed, in all these cases, as JSON.

## If clause only

::: if true

This paragraph **will** be visible.

:::

## If and else clauses

::: if false

This paragraph will not be visible.

::: else {json5}

This paragraph **will** be visible.

:::

## If, elif and else clauses

::: if true

This paragraph **will** be visible.

::: elif false

This paragraph will not be visible.

:::

## If and elif clauses

::: if true

This paragraph will not be visible.

::: elif true

This paragraph **will** be visible.

::: elif false

This paragraph will not be visible.

:::

# Language specified

Examples where the language is specified.

## On first clause only

::: if 1 + 1 > 0 {calc}

This paragraph **will** be visible.

:::

## On some clauses

::: if 1 {calc}

This paragraph will not be visible.

::: elif 1 == 1 {python}

This paragraph **will** be visible.

:::

## On all clauses

::: if 1 == 1 {python}

This paragraph **will** be visible.

::: elif 1 == 0 {calc}

This paragraph will not be visible.

:::

# Truthiness of clause expressions

Stencila uses the [same criteria as Python](https://www.pythonmorsels.com/truthiness/) for determining the truthiness of the value of an evaluated expression in a `if` / `elif` clause:

- `null` and `false` are falsy
- integers and numbers equal to zero are falsy
- strings, arrays, objects, and datatables that are empty are falsy
- all other values are thruthy

The following examples illustrate how truthyness is determined for common value types. Only the last clause should be visible.

## `Null`

Nulls are falsy.

::: if null

This paragraph will not be visible.

::: else {json5}

This paragraph **will** be visible.

:::

## `Boolean`

Booleans are inherently truthy or falsy.

::: if false

This paragraph will not be visible.

::: elif true

This paragraph **will** be visible.

:::

## `Integer`

Integers equal to zero are falsy, otherwise they are truthy (including negative values).

::: if 0

This paragraph will not be visible.

::: elif -1

This paragraph **will** be visible.

:::

## `Number`

Numbers equal to zero are falsy, otherwise they are truthy (including negative values).

::: if 0.0

This paragraph will not be visible.

::: elif 1.0

This paragraph **will** be visible.

:::

## `String`

Strings that are empty are falsy, otherwise they are truthy.

::: if ""

This paragraph will not be visible.

::: elif "Hello"

This paragraph **will** be visible.

:::

## `Array`

Arrays that are empty are falsy, otherwise they are truthy.

::: if [1,2,3]

This paragraph **will** be visible.

::: elif []

This paragraph will not be visible.

:::

## `Object`

Objects that are empty are falsy, otherwise they are truthy.

::: if `{}`

This paragraph will not be visible.

::: elif `{a:0}`

This paragraph **will** be visible.

:::

## `Datatable`

Expressions that evaluate to datatables (e.g. R `data.frame`s and Pandas `DataFrame`s) are falsy if they have no rows, otherwise they are truthy.

**This is currently not working properly because, at present, R `data.frame`s are not correctly decoded to a `Datatable`**

::: if data.frame() {r}

This paragraph will not be visible because the data frame has no rows.

::: elif mtcars {r}

This paragraph **will** be visible because the mtcars data frame has rows.

:::

# Errors in clause expressions

If there is an error in a clause expression it is displayed next to it.

::: if foo {json5}

This will not be visible because the expression errored.

::: elif bar {py}

This will not be visible because the expression errored.

::: elif baz {r}

This will not be visible because the expression errored.

::: elif quax {js}

This will not be visible because the expression errored.

::: elif beep {sql}

This will not be visible because the expression errored.

::: elif boop {prql}

This will not be visible because the expression errored.

::: else

This **will** be visible and the other clauses will have error messages.

:::

# Nesting

If blocks can be nested within other blocks.

## With another `if` block

::: if true

::: if true

:::

:::

## Within a `for` block

::: for item in [1,2,3]

::: if item == 1

This paragraph will only be visible for the first item.

:::

:::
