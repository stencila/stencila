/**
 * HTML element attributes that are used to represent properties of `struct`s.
 *
 * These are mappings from Stencila Schema property names to
 * [HTML attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes)
 * used to represent them
 */
export const STRUCT_ATTRIBUTES: Record<string, string> = {
  // Entity
  id: 'id',
  // TableCell
  rowspan: 'rowspan',
  colspan: 'colspan',
  // MediaObject
  contentUrl: 'src',
  // Parameter
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
}
