# Code Executable

**Base type for executable code nodes (i.e. `CodeChunk` and `CodeExpression`).**

Adds properties to the base `Code` node type that are necessary, or useful, for executable code. Also makes the `programmingLanguage` property required since that is necessary to be able to execute it.

## Properties

| Name                    | `@id`                                                                         | Type                                                                                                               | Description                                                                             | Inherited from                      |
| ----------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- | ----------------------------------- |
| **programmingLanguage** | [schema:programmingLanguage](https://schema.org/programmingLanguage)          | string                                                                                                             | The programming language of the code.                                                   | [Code](Code.md)                     |
| **text**                | [schema:text](https://schema.org/text)                                        | string                                                                                                             | The text of the code.                                                                   | [Code](Code.md)                     |
| codeDependencies        | [stencila:codeDependencies](https://schema.stenci.la/codeDependencies.jsonld) | Array of ([CodeChunk](CodeChunk.md) _or_ [Parameter](Parameter.md))                                                | The upstream dependencies of the code. See note [1](#notes).                            | [CodeExecutable](CodeExecutable.md) |
| codeDependents          | [stencila:codeDependents](https://schema.stenci.la/codeDependents.jsonld)     | Array of ([CodeChunk](CodeChunk.md) _or_ [CodeExpression](CodeExpression.md))                                      | The downstream dependents of the code. See note [2](#notes).                            | [CodeExecutable](CodeExecutable.md) |
| compileDigest           | [stencila:compileDigest](https://schema.stenci.la/compileDigest.jsonld)       | string                                                                                                             | A digest of the content, semantics and dependencies of the node.                        | [CodeExecutable](CodeExecutable.md) |
| errors                  | [stencila:errors](https://schema.stenci.la/errors.jsonld)                     | Array of [CodeError](CodeError.md)                                                                                 | Errors when compiling (e.g. syntax errors) or executing the chunk.                      | [CodeExecutable](CodeExecutable.md) |
| executeCount            | [stencila:executeCount](https://schema.stenci.la/executeCount.jsonld)         | integer                                                                                                            | A count of the number of times that the node has been executed. See note [3](#notes).   | [CodeExecutable](CodeExecutable.md) |
| executeDigest           | [stencila:executeDigest](https://schema.stenci.la/executeDigest.jsonld)       | string                                                                                                             | The `compileDigest` of the node when it was last executed.                              | [CodeExecutable](CodeExecutable.md) |
| executeDuration         | [stencila:executeDuration](https://schema.stenci.la/executeDuration.jsonld)   | number                                                                                                             | Duration in seconds of the last execution of the code.                                  | [CodeExecutable](CodeExecutable.md) |
| executeEnded            | [stencila:executeEnded](https://schema.stenci.la/executeEnded.jsonld)         | [Date](Date.md)                                                                                                    | The date-time that the the last execution of the code ended.                            | [CodeExecutable](CodeExecutable.md) |
| executeRequired         | [stencila:executeRequired](https://schema.stenci.la/executeRequired.jsonld)   | 'No', 'NeverExecuted', 'SemanticsChanged', 'DependenciesChanged', 'DependenciesFailed'                             | Whether, and why, a node requires execution or re-execution. See note [4](#notes).      | [CodeExecutable](CodeExecutable.md) |
| executeStatus           | [stencila:executeStatus](https://schema.stenci.la/executeStatus.jsonld)       | 'Scheduled', 'ScheduledPreviouslyFailed', 'Running', 'RunningPreviouslyFailed', 'Succeeded', 'Failed', 'Cancelled' | Status of the most recent, including any current, execution of the code.                | [CodeExecutable](CodeExecutable.md) |
| id                      | [schema:id](https://schema.org/id)                                            | string                                                                                                             | The identifier for this item.                                                           | [Entity](Entity.md)                 |
| mediaType               | [schema:encodingFormat](https://schema.org/encodingFormat)                    | string                                                                                                             | Media type, typically expressed using a MIME format, of the code. See note [5](#notes). | [Code](Code.md)                     |
| meta                    | [stencila:meta](https://schema.stenci.la/meta.jsonld)                         | object                                                                                                             | Metadata associated with this item.                                                     | [Entity](Entity.md)                 |

## Notes

1. **codeDependencies** : Note that this excludes `CodeExpression` nodes since they should not have side effects (e.g. assigning variables) that would cause another node to be dependent upon them.
2. **codeDependents** : Note that in comparison to `codeDependencies`, this property does not allow for `Parameter` dependents (because parameters are never dependent upon others).
3. **executeCount** : Intended to increment with each successive execution of the node, including across sessions. Note that this differs to the `execution_count` in Jupyter Notebook format which is the "code cell's prompt number" and which resets at the start of each new session.
4. **executeRequired** : Derived from a comparison of `compileDigest` and `executeDigest` and the `executeStatus` of dependencies. `No`: no re-execution is required, the semantics of the code and its dependencies has not changed since it was last executed. `NeverExecuted`: execution is required because the code has never been executed (or any previous execution was not persisted in its state). `SemanticsChanged`: re-execution is required because the semantics of the code has changed since it was last executed. `DependenciesChanged`: the semantics of one or more dependencies (including transitive dependencies) changed since it was last executed. `DependenciesFailed`: one or more dependencies (including transitive dependencies) failed when it was last executed.
5. **mediaType** : This property allows the differentiation of formats using the same programming language or variants of a programming language. An example is using `programmingLanguage` "json" and `encodingFormat` "application/ld+json" for JSON-LD code examples.

## Related

- Parent: [Code](Code.md)
- Descendants: [CodeChunk](CodeChunk.md), [CodeExpression](CodeExpression.md)

## Available as

- [JSON-LD](https://schema.stenci.la/CodeExecutable.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/CodeExecutable.schema.json)
- Python [`class CodeExecutable`](https://stencila.github.io/schema/python/docs/types.html#schema.types.CodeExecutable)
- TypeScript [`interface CodeExecutable`](https://stencila.github.io/schema/ts/docs/interfaces/codeexecutable.html)
- R [`class CodeExecutable`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct CodeExecutable`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.CodeExecutable.html)

## Source

This documentation was generated from [CodeExecutable.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/CodeExecutable.schema.yaml).
