/**
 * Get the computed value of a CSS custom property from an element
 *
 * CSS custom properties are stored as literal strings and don't get computed
 * until used in a real CSS property. This function applies the variable to a
 * temporary element to force computation of calc() and var() expressions.
 *
 * For dimensional values (lengths, sizes), it uses the 'fontSize' property. For
 * colors (color-mix, rgb, hsl, etc.), the browser pre-computes them, so no temp
 * element needed.
 *
 * @param element - The element to read the CSS variable from
 * @param propertyName - The CSS custom property name (with or without leading
 * --)
 * @param defaultValue - Default value if property is not found or empty
 * @returns The computed CSS custom property value with calc() and var()
 * evaluated
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
  if (!value) return defaultValue

  // Colors (color-mix, rgb, hsl, etc.) are already computed by the browser
  // No need to create a temp element for these
  if (value.startsWith('color-mix(') || value.startsWith('rgb') || value.startsWith('hsl')) {
    return value
  }

  // If the value contains calc() or var(), we need to compute it using a temp element
  // We use 'font-size' property which reliably computes dimensional values (px, rem, em, %)
  if (value.includes('calc(') || value.includes('var(')) {
    const temp = document.createElement('div')
    temp.style.position = 'absolute'
    temp.style.visibility = 'hidden'
    temp.style.fontSize = `var(${normalizedName})`

    element.appendChild(temp)
    const computed = getComputedStyle(temp).fontSize
    element.removeChild(temp)

    return computed
  }

  return value
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
