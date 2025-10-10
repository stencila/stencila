/**
 * Deep merge utility for plot configurations
 *
 * Recursively merges two objects, handling nested objects, arrays, and primitives.
 *
 * @param base - The base object
 * @param override - The override object
 * @param strategy - Merge strategy:
 *   - 'override-wins' (default): override values take precedence at each level
 *   - 'base-wins': base values take precedence at each level
 * @returns Merged object
 *
 * @example
 * // override-wins (default)
 * deepMerge(
 *   { a: 1, b: { x: 1, y: 2 } },
 *   { b: { x: 3 }, c: 4 }
 * )
 * // => { a: 1, b: { x: 3, y: 2 }, c: 4 }
 *
 * @example
 * // base-wins
 * deepMerge(
 *   { a: 1, b: { x: 1, y: 2 } },
 *   { b: { x: 3 }, c: 4 },
 *   'base-wins'
 * )
 * // => { a: 1, b: { x: 1, y: 2 }, c: 4 }
 */
export function deepMerge(
  base: Record<string, unknown>,
  override: Record<string, unknown>,
  strategy: 'override-wins' | 'base-wins' = 'override-wins'
): Record<string, unknown> {
  // If override is null/undefined, return base
  if (override === null || override === undefined) {
    return base
  }

  // If base is null/undefined, return override
  if (base === null || base === undefined) {
    return override
  }

  // If either is not an object (primitives), apply strategy
  if (typeof base !== 'object' || typeof override !== 'object') {
    return strategy === 'override-wins' ? override : base
  }

  // Handle arrays - take the winner based on strategy (no deep merge for arrays)
  if (Array.isArray(base) || Array.isArray(override)) {
    return strategy === 'override-wins' ? override : base
  }

  // Deep merge objects
  const result: Record<string, unknown> = { ...base }

  for (const key in override) {
    if (Object.prototype.hasOwnProperty.call(override, key)) {
      const baseValue = base[key]
      const overrideValue = override[key]

      // Check if both values are plain objects (not arrays, not null)
      const baseIsObject =
        typeof baseValue === 'object' &&
        baseValue !== null &&
        !Array.isArray(baseValue)
      const overrideIsObject =
        typeof overrideValue === 'object' &&
        overrideValue !== null &&
        !Array.isArray(overrideValue)

      if (baseIsObject && overrideIsObject) {
        // Recursively merge nested objects
        result[key] = deepMerge(
          baseValue as Record<string, unknown>,
          overrideValue as Record<string, unknown>,
          strategy
        )
      } else if (strategy === 'override-wins') {
        // Override wins: use override value
        result[key] = overrideValue
      } else {
        // Base wins: only use override if base doesn't have this key
        if (!(key in base)) {
          result[key] = overrideValue
        }
        // else keep base value (already in result from spread)
      }
    }
  }

  return result
}
