/**
 * Fallback to a default value if the value is `undefined`.
 *
 * This function is an elint-complaint-free version of:
 *
 * ```ts
 * possiblyUndefinedValue || fallbackValue
 * ```
 *
 * Use it like this:
 *
 * ```ts
 * fallback(possiblyUndefinedValue, fallbackValue)
 * ```
 *
 * @param value The value to return if it is defined
 * @param fallback The value to return if `value` is `undefined.
 */
export function fallback<T>(value: T | undefined, fallback: T): T {
  return value === undefined ? fallback : value
}
