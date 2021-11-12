/**
 * HTML element attributes that are used to represent properties of `struct`s.
 *
 * These are [HTML attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes)
 * that are also Stencila Schema property names.
 */
export const STRUCT_ATTRIBUTES = ['id', 'value', 'rowspan', 'colspan']

/**
 * HTML element attributes that are aliases for properties of `struct`s.
 */
export const STRUCT_ATTRIBUTE_ALIASES: Record<string, string> = {
  contentUrl: 'src',
}
