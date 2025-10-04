/**
 * Convert CSS color to hex format
 *
 * Takes any valid CSS color (named colors, rgb(), rgba(), css variables, color-mix(), oklch(), etc.)
 * and converts it to a hex color code suitable for libraries that require hex format.
 *
 * @param color - CSS color value (e.g., 'red', 'rgb(255, 0, 0)', 'var(--primary-color)')
 * @returns Hex color code (e.g., '#ff0000') or the computed color if conversion fails
 */
export function colorToHex(color: string): string {
  // Return as-is if already hex
  if (color.startsWith('#')) {
    return color
  }

  // Create a temporary element to compute the color
  const temp = document.createElement('div')
  temp.style.color = color
  document.body.appendChild(temp)
  const computed = getComputedStyle(temp).color
  document.body.removeChild(temp)

  // Try to match rgb(a) format
  const rgbMatch = computed.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/)
  if (rgbMatch) {
    const [, r, g, b] = rgbMatch
    return '#' + [r, g, b].map((x) => parseInt(x).toString(16).padStart(2, '0')).join('')
  }

  // Try to match color(srgb ...) format
  const srgbMatch = computed.match(/color\(srgb\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)/)
  if (srgbMatch) {
    const [, r, g, b] = srgbMatch
    const toHexPart = (val: string) => Math.round(parseFloat(val) * 255).toString(16).padStart(2, '0')
    return '#' + [r, g, b].map(toHexPart).join('')
  }

  // For oklch and other modern formats, use canvas to convert
  // Canvas always renders to RGB regardless of input format
  if (computed.startsWith('oklch') || computed.startsWith('lab') || computed.startsWith('lch')) {
    try {
      const canvas = document.createElement('canvas')
      canvas.width = 1
      canvas.height = 1
      const ctx = canvas.getContext('2d')
      if (ctx) {
        ctx.fillStyle = computed
        ctx.fillRect(0, 0, 1, 1)
        const imageData = ctx.getImageData(0, 0, 1, 1)
        const [r, g, b] = imageData.data
        return '#' + [r, g, b].map((x) => x.toString(16).padStart(2, '0')).join('')
      }
    } catch (e) {
      // Canvas conversion failed, fall through to fallback
    }
  }

  // Fallback: return computed value or original
  return computed || color
}
