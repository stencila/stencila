/**
 * TOC generation from heading elements
 *
 * Generates table of contents HTML from stencila-heading elements
 * in the main content, matching the server-rendered TOC structure.
 */

/** Heading info extracted from the document */
interface HeadingInfo {
  id: string
  level: number
  text: string
}

/**
 * Extract headings from main content element
 *
 * @param main - Main content element to search
 * @param maxDepth - Maximum heading level to include (1-6)
 * @returns Array of heading info objects
 */
function extractHeadings(main: HTMLElement, maxDepth: number): HeadingInfo[] {
  const headings: HeadingInfo[] = []

  // Query stencila-heading elements with level and id attributes
  const elements = main.querySelectorAll('stencila-heading[level][id]')

  for (const el of elements) {
    const level = parseInt(el.getAttribute('level') ?? '1', 10)
    const id = el.getAttribute('id')

    // Skip if level exceeds maxDepth or no id
    if (level > maxDepth || !id) {
      continue
    }

    // Get text content from the heading
    const text = el.textContent?.trim() ?? ''
    if (text) {
      headings.push({ id, level, text })
    }
  }

  return headings
}

/**
 * Build nested TOC HTML from flat heading list
 *
 * Creates a nested ul/li structure matching the server-rendered TOC:
 * - ul[role="tree"] - root TOC list
 * - li[role="treeitem"] - TOC items
 * - a.toc-link - links to heading anchors
 * - ul[role="group"] - nested groups for child headings
 */
function buildTocHtml(headings: HeadingInfo[]): string {
  if (headings.length === 0) {
    return ''
  }

  const lines: string[] = []
  const stack: number[] = [] // Track open list levels

  lines.push('<ul role="tree" class="toc-list">')
  stack.push(0) // Root level marker

  for (let i = 0; i < headings.length; i++) {
    const heading = headings[i]
    const nextHeading = headings[i + 1]

    // Close lists for higher levels
    while (stack.length > 1 && stack[stack.length - 1] >= heading.level) {
      lines.push('</li>')
      lines.push('</ul>')
      stack.pop()
    }

    // Check if next heading is a child (deeper level)
    const hasChildren = nextHeading && nextHeading.level > heading.level

    // Open new list item (with aria-expanded if has children)
    if (hasChildren) {
      lines.push(`<li role="treeitem" aria-expanded="true">`)
    } else {
      lines.push(`<li role="treeitem">`)
    }
    lines.push(`<a class="toc-link" href="#${heading.id}" data-level="${heading.level}">${escapeHtml(heading.text)}</a>`)

    if (hasChildren) {
      lines.push('<ul role="group" class="toc-list">')
      stack.push(heading.level)
    } else {
      lines.push('</li>')
    }
  }

  // Close remaining open lists
  while (stack.length > 1) {
    lines.push('</li>')
    lines.push('</ul>')
    stack.pop()
  }

  lines.push('</ul>')

  return lines.join('\n')
}

/**
 * Escape HTML special characters
 */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

/**
 * Generate TOC HTML from headings in main content
 *
 * @param main - Main content element containing headings
 * @param maxDepth - Maximum heading level to include (default: 3)
 * @returns Generated TOC HTML string
 */
export function generateTocFromHeadings(main: HTMLElement, maxDepth = 3): string {
  const headings = extractHeadings(main, maxDepth)
  return buildTocHtml(headings)
}
