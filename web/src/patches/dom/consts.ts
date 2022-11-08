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
  guessLanguage: 'guess-language',
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
  name: 'name',
  label: 'label',
  default: 'default',
  value: 'value',
  // Button
  isDisabled: 'is-disabled',
  // EnumValidator,
  values: 'values',
  // IntegerValidator and NumberValidator
  minimum: 'minimum',
  maximum: 'maximum',
  exclusiveMinimum: 'exclusive-minimum',
  exclusiveMaximum: 'exclusive-maximum',
  multipleOf: 'multiple-of',
  // StringValidator
  minLength: 'min-length',
  maxLength: 'max-length',
  pattern: 'pattern',
  // For
  symbol: 'symbol',
  text: 'text',
  // If
  isActive: 'is-active',
  // CodeError
  errorType: 'error-type',
}
