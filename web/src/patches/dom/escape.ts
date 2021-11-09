/**
 * Escape a string so that it is safe to use as the inner text content
 * of a HTML element.
 */
export function escapeHtml(unsafe: string): string {
  const text = document.createTextNode(unsafe)
  const temp = document.createElement('p')
  temp.appendChild(text)
  return temp.innerHTML
}

/**
 * Unescape HTML.
 *
 * Should be the inverse of `escapeHtml`.
 */
export function unescapeHtml(html: string): string {
  const doc = new DOMParser().parseFromString(html, 'text/html')
  return doc.documentElement.textContent ?? ''
}

/**
 * Escape a string so that it is safe to use as the text content
 * of a HTML attribute.
 */
export function escapeAttr(unsafe: string): string {
  return unsafe.replace(/["']/g, function (m) {
    switch (m) {
      case '"':
        return '&quot;'
      default:
        return '&#039;'
    }
  })
}

/**
 * Unescape an attribute text.
 *
 * Should be the inverse of `escapeAttr`.
 */
export function unescapeAttr(text: string): string {
  return text.replace(/&quot;/g, '"').replace(/&#039;/g, "'")
}
