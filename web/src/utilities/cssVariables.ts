/**
 * Get the computed value of a CSS custom property from an element
 *
 * @param element - The element to read the CSS variable from
 * @param propertyName - The CSS custom property name (with or without leading --)
 * @param defaultValue - Default value if property is not found or empty
 * @returns The computed CSS custom property value
 */
export function getCSSVariable(
  element: Element,
  propertyName: string,
  defaultValue = ''
): string {
  const computedStyle = getComputedStyle(element)
  const normalizedName = propertyName.startsWith('--')
    ? propertyName
    : `--${propertyName}`

  const value = computedStyle.getPropertyValue(normalizedName).trim()
  return value || defaultValue
}

/**
 * Get multiple CSS custom properties as an object
 *
 * @param element - The element to read CSS variables from
 * @param propertyNames - Object mapping result keys to CSS property names
 * @returns Object with computed CSS custom property values
 */
export function getCSSVariables(
  element: Element,
  propertyNames: Record<string, string>
): Record<string, string> {
  const result: Record<string, string> = {}

  for (const [key, propertyName] of Object.entries(propertyNames)) {
    result[key] = getCSSVariable(element, propertyName)
  }

  return result
}

/**
 * Check if a CSS custom property indicates a feature should be visible
 *
 * Treats 'none', 'hidden', 'false' as false, everything else as true
 *
 * @param element - The element to read the CSS variable from
 * @param propertyName - The CSS custom property name
 * @param defaultVisible - Default visibility if property is not found
 * @returns Whether the feature should be visible
 */
export function isCSSFeatureVisible(
  element: Element,
  propertyName: string,
  defaultVisible = true
): boolean {
  const value = getCSSVariable(element, propertyName).toLowerCase()

  if (!value) return defaultVisible

  const hiddenValues = ['none', 'hidden', 'false', '0']
  return !hiddenValues.includes(value)
}
