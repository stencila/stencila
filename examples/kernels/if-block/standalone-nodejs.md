::: if null {nodejs}

`null` is falsy

::: elif false

`false` is falsy

::: elif -1

Non-positive integers are falsy

::: elif 0

Non-positive unsigned integers are falsy

::: elif -1.0

Non-positive numbers are falsy

::: elif ""

Empty strings are falsy

::: elif []

Empty arrays are falsy

::: elif `{}`

Empty objects are falsy

::: else

Evaluated in Node.js {{ process.version }}

:::
