/**
 * Escape a string to be used in HTML
 */
export function escapeHtml(html: string) {
  const div = document.createElement('div')
  div.textContent = html
  return div.innerHTML
}
