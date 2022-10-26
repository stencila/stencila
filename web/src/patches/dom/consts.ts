/**
 * HTML element attributes that are used to represent properties of `struct`s.
 *
 * These are mappings from Stencila Schema property names to the
 * [HTML attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes)
 * used to represent them.
 */
export const STRUCT_ATTRIBUTES: Record<string, string> = {
  // Entity
  id: 'id',
  // CodeChunk and CodeExpression
  programmingLanguage: 'programming-language',
  compileDigest: 'compile-digest',
  executeDigest: 'execute-digest',
  executeRequired: 'execute-required',
  executeKernel: 'execute-kernel',
  executeStatus: 'execute-status',
  executeCount: 'execute-count',
  // TableCell
  rowspan: 'rowspan',
  colspan: 'colspan',
  // MediaObject
  contentUrl: 'src',
  // Parameter
  default: 'placeholder',
  value: 'value',
  // NumberValidator
  type: 'type',
  minimum: 'min',
  maximum: 'max',
  multipleOf: 'step',
  // StringValidator
  minLength: 'minlength',
  maxLength: 'maxlength',
  pattern: 'pattern',
  // For
  symbol: 'symbol',
  text: 'text',
  // If
  isActive: 'is-active',
  guessLanguage: 'guess-language',
  // CodeError
  errorType: 'error-type',
}
