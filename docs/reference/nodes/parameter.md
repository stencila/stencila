.

# About

This is technical reference documentation for the Stencila `Parameter` node type. For less technical documentation, please see the guides and tutorials. For more technical documentation, please refer to the Stencila Schema [documentation](../schema/Parameter.md). As with the other reference documentation, this page partly serves to test functionality so contains many, often similar examples.

# Name and label

Parameters always have a `name`. When a parameter is executed, the value of the parameter will be `set` in the `store` kernel using the name.

Parameters can also have a label that will appear before the input and provide accessible access to it:

&[con]{num min=0.0 max=100.0 label="Concentration (mg/l)"}

Labels are designed to be inline so they can be part of the flow, while still providing for accessibility.

# Validators

Parameters optionally have a `validator`. When the value of a parameter is changed it is checked against the validator. The validator also determines the type of user input for the parameter.

## No validator

If no validator is specified, then the input will default to a text input, and the `value` of the parameter will be a string e.g.

&[par1]

## Enum validator

If the validator is an `EnumValidator`, then the input will be a dropdown listing the possible `values` for the `value` of the parameter e.g.

&[enum_1]{enum vals=["apple","pear","orange"]}

## Boolean validator

Boolean validators allow the user to set the parameter to be either `true` or `false`. e.g.

&[bool_1]{bool}

## Integer validator

Integer validators have five optional constraints:

- `minimum`: the minimum allowed valid value (i.e. the value must be greater than, to equal to, this)
- `exclusiveMinimum`: the exclusive minimum valid value (i.e. the value must be greater than this)
- `maximum`: the maximum allowed valid value (i.e. the value must be less than, to equal to, this)
- `exclusiveMaximum`: the exclusive maximum valid value (i.e. the value must be less than this)
- `multipleOf`: the value must be a multiple of this

### No constraints

When an integer validator has no constraints, users can enter and numbers into the parameter's input e.g.

&[int]{int}

### With `minimum`

This parameter has a integer validator with a `minimum` of 5. Try entering a lower value.

&[int_with_min]{int min=5.0}

### With `exclusiveMinimum`

This parameter has a integer validator with an `exclusiveMinimum` of 5. Try entering a value less than or equal to that.

&[int_with_exclusive_min]{int exmin=5.0}

### With `minimum` and `maximum`

If the validator has both a `minimum` (or an `exclusiveMinimum`) and a `maximum` (or an `exclusiveMaximum`), then the parameter input will be a slider with a step size of `multipleOf` (or 1, if not specified) e.g.

### With `minimum`, `maximum`, and `multipleOf`

If ...

&[int_with_min_max_and_mult]{int min=1.0 max=10.0 mult=2.0}

## Number validator

&[num_1]{num min=12.0 max=100.0}

Number validators have the same

&[num_2]{num max=10.0}

## String validator

String validators have three optional constraints:

- `minLength`: the minimum length of the string
- `maxLength`: the maximum length of the string
- `pattern`: a regular expression that the string should match

### With no constraints

When a string validator has no constraints, users can enter any characters into the input e.g.

&[string]{str}

### With `minLength`

This parameter has a string validator with a `minLength` of 3. Try entering fewer characters.

&[string_min]{str min=3}

### With `maxLength`

When `maxLength` is specified, the user will not be able to type in more characters than the maximum.

This parameter has a string validator with a `maxLength` of 5. Try entering more characters.

&[string_max]{str max=5}

### With `minLength` and `maxLength`

When using both `minLength` and `maxLength` make sure that the former is less than, or equal to, the latter.

&[string_min_max]{str min=3 max=5}

### With `pattern`

This parameter has a string validator with a `pattern` only allowing the characters "a", "b", or "c". Try entering different characters.

&[string_pattern]{str pattern="[abc]+"}

### With `minLength`, `maxLength`, and `pattern`

When using `pattern` with either `minLength` and/or `maxLength` make sure that they are consistent i.e. that the length contraints allow the pattern constraint to be met and vice vera.

This parameter has a string validator with all three constraints specified.

&[string_min_max]{str min=3 max=5 pattern="[abc]+"}

## Date validator

&[date_1]{date}

## Time validator

&[time_1]{time}

## DateTime validator

&[datetime_1]{datetime}

## Timestamp validator

&[timestamp_1]

## Duration validator

&[duration_1]

# Content type

A `Parameter` node is inline content. As such, it can appear in many places in a document. It is left to document authors to decide how best to layout parameters within the surrounding content. Some examples follow,

## Within a paragraph

Parameters can appear as part of a content of a paragraph, e.g. &[in_para_1]{bool} and &[in_para_2]{num}.

## Within a list

Parameters can be within a paragraph within a list item e.g.

- &[par_in_list_1]{int}
- &[par_in_list_2]{num}
- &[par_in_list_3]{str}

## Within a table cell

Another way to layout parameters is to put them into a table. For example, here's a table that combines three `Parameter`s with a `CodeExpression` to calculate the volume of an object.

| Description |                                     |
| ----------- | ----------------------------------- |
| Width       | &[width]{num min=0.0 max=1.0}       |
| Height      | &[height]{num min=0.0 max=1.0}      |
| Depth       | &[depth]{num min=0.0 max=1.0}       |
| Volume      | `width * height * depth`{calc exec} |

## Within a `Division` or `Span`

If you need to layout your parameters in some other way you can use the styling node types `Division` and `Span`. For example, this division lays out four parameters in a 2 x 3 grid.

::: grid grid-cols-2 gap-2

&[A1]{int min=1.0 max=100.0}

&[B1]{int min=1.0 max=100.0}

&[A2]{int min=1.0 max=100.0}

&[B2]{int min=1.0 max=100.0}

`A1 + A2`{calc exec}

`B2 + B2`{calc exec}

:::
