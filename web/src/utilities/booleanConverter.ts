/**
 * Convert a string attribute on a custom element to a boolean value
 *
 * Needed because the generated DOM HTML represents boolean properties
 * of nodes as string attributes (e.g is-active="false") rather than
 * HTML's (and LitElement's) presence/absence of the attribute.
 */
export const booleanConverter = (attr: string) => attr == 'true'
