/**
 * Shared event contract for property input components.
 *
 * Field components (persistent id, programming language, boolean) and the
 * popovers that consume them all reference this single constant so siblings are
 * not coupled to one another's modules just to agree on an event name.
 */

/**
 * Bubbling, composed event dispatched by a property input when its value
 * changes. The new value is carried in `detail.value`.
 */
export const EDIT_PROPERTY_VALUE_CHANGE_EVENT = 'edit-property-value-change'
