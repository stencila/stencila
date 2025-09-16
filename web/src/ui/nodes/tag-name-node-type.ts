import { NodeType } from '@stencila/types'

/**
 * Convert a NodeType to its corresponding custom element tag name
 *
 * Converts PascalCase NodeType to kebab-case with 'stencila-' prefix
 * e.g. 'CodeBlock' -> 'stencila-code-block'
 */
export const nodeTypeToTagName = (nodeType: NodeType): string => {
  const kebabCase = nodeType
    .replace(/([A-Z])/g, '-$1')
    .toLowerCase()
    .replace(/^-/, '')

  return `stencila-${kebabCase}`
}

/**
 * Convert a custom element tag name to its corresponding NodeType
 *
 * Strips 'stencila-' prefix and converts kebab-case to PascalCase
 * e.g. 'stencila-code-block' -> 'CodeBlock'
 */
export const tagNameToNodeType = (tagName: string): NodeType => {
  const lowerCase = tagName.toLowerCase()

  if (!lowerCase.startsWith('stencila-')) {
    return 'Null' as NodeType
  }

  const withoutPrefix = lowerCase.replace(/^stencila-/, '')
  const pascalCase = withoutPrefix
    .split('-')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join('')

  return pascalCase as NodeType
}
