/**
 * Convert CSS color to hex format
 *
 * Takes any valid CSS color (named colors, rgb(), rgba(), css variables, etc.)
 * and converts it to a hex color code suitable for libraries that require hex format.
 *
 * @param color - CSS color value (e.g., 'red', 'rgb(255, 0, 0)', 'var(--primary-color)')
 * @returns Hex color code (e.g., '#ff0000') or the computed color if conversion fails
 */
export function colorToHex(color: string): string {
  // Create a temporary element to compute the color
  const temp = document.createElement('div')
  temp.style.color = color
  document.body.appendChild(temp)
  const computed = getComputedStyle(temp).color
  document.body.removeChild(temp)

  // Convert rgb(a) to hex
  const match = computed.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/)
  if (match) {
    const [, r, g, b] = match
    return '#' + [r, g, b].map((x) => parseInt(x).toString(16).padStart(2, '0')).join('')
  }

  // Return as-is if already hex or named color
  return color.startsWith('#') ? color : computed
}
