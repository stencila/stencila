::: if NULL {r}

`NULL` is falsy

::: elif NA

`NA` is falsy

::: elif FALSE

`FALSE` is falsy

::: elif -1

Non-positive integers are falsy

::: elif 0

Non-positive unsigned integers are falsy

::: elif -1.0

Non-positive numbers are falsy

::: elif ""

Empty strings are falsy

::: elif numeric()

Empty vectors are falsy

::: elif list()

Empty lists are falsy

::: else

Evaluated in {{ version$version.string }}

:::
