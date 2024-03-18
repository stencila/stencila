import type { NodeType } from '@stencila/types'
import tailwindConfig from 'tailwindcss/defaultConfig'
import resolveConfig from 'tailwindcss/resolveConfig'

export type NodeTypeUI = {
  title?: string
  icon?: string
  iconLibrary?: string
  colour?: string
  borderColour?: string
}

const colours = resolveConfig(tailwindConfig).theme.colors

const DEFAULT_ICON = 'box'
const DEFAULT_ICON_LIBRARY = 'default'
const DEFAULT_COLOUR = colours.blue[100]
const DEFAULT_BORDER_COLOUR = colours.blue[200]

const stencilaIcon = (icon: string) => ({
  icon,
  iconLibrary: 'stencila',
})

const shoelaceIcon = (icon: string) => ({
  icon,
  iconLibrary: 'default',
})

const nodeColours = (name: string) => ({
  colour: colours[name][100],
  borderColour: colours[name][200],
})

// prettier-ignore
const nodeTypeUIMap: Partial<Record<NodeType, NodeTypeUI>> = {
  Admonition:       { ...stencilaIcon('admonition'),           ...nodeColours('violet')},
  Array:            { ...stencilaIcon('array'),                ...nodeColours('gray')},
  Boolean:          { ...shoelaceIcon('toggle-off'),           ...nodeColours('gray')},
  Claim:            { ...shoelaceIcon('postage'),              ...nodeColours('orange')},
  CodeBlock:        { ...shoelaceIcon('braces'),               ...nodeColours('fuchsia')},
  CodeChunk:        { ...stencilaIcon('code-chunk'),           ...nodeColours('fuchsia')},
  DeleteBlock:      { ...shoelaceIcon('dash-circle'),          ...nodeColours('red')},
  Figure:           { ...shoelaceIcon('image'),                ...nodeColours('gray')},
  ForBlock:         { ...stencilaIcon('for-block'),            ...nodeColours('pink')},
  Heading:          { ...stencilaIcon('heading'),              ...nodeColours('gray')},
  IfBlock:          { ...stencilaIcon('if-block'),             ...nodeColours('amber')},
  ImageObject:      { ...shoelaceIcon('image-alt'),            ...nodeColours('slate'),  title: 'Image' },
  InsertBlock:      { ...shoelaceIcon('plus-circle'),          ...nodeColours('lime')},
  InstructionBlock: { ...stencilaIcon('instruction-block'),    ...nodeColours('blue')},
  Integer:          { ...shoelaceIcon('123'),                  ...nodeColours('gray')},
  List:             { ...stencilaIcon('list'),                 ...nodeColours('gray')},
  MathBlock:        { ...stencilaIcon('math-block'),           ...nodeColours('fuchsia')},
  Number:           { ...shoelaceIcon('123'),                  ...nodeColours('gray')},
  Paragraph:        { ...stencilaIcon('paragraph'),            ...nodeColours('gray')},
  Object:           { ...shoelaceIcon('boxes'),                ...nodeColours('gray')},
  QuoteBlock:       { ...shoelaceIcon('quote'),                ...nodeColours('yellow')},
  ReplaceBlock:     { ...stencilaIcon('replace-block'),        ...nodeColours('orange')},
  Section:          { ...shoelaceIcon('square'),               ...nodeColours('fuchsia')},
  String:           { ...shoelaceIcon('alphabet'),             ...nodeColours('gray')},
  StyledBlock:      { ...shoelaceIcon('palette'),              ...nodeColours('pink')},
  Table:            { ...stencilaIcon('table'),                ...nodeColours('gray')},
  ThematicBreak:    { ...shoelaceIcon('hr'),                   ...nodeColours('slate')},
}

/**
 * Get the UI specifications for a node type
 */
export const nodeUi = (nodeType: NodeType): Required<NodeTypeUI> => {
  const ui = nodeTypeUIMap[nodeType]
  return {
    title: ui?.title ?? nodeType.replace(/([A-Z])/g, ' $1').trim(),
    icon: ui?.icon ?? DEFAULT_ICON,
    iconLibrary: ui?.iconLibrary ?? DEFAULT_ICON_LIBRARY,
    colour: ui?.colour ?? DEFAULT_COLOUR,
    borderColour: ui?.borderColour ?? DEFAULT_BORDER_COLOUR,
  }
}

/**
 * Get the title for a node type
 */
export const nodeTitle = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.title ?? nodeType.replace(/([A-Z])/g, ' $1').trim()

/**
 * Get the (background) colour for a node type
 */
export const nodeColour = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.colour ?? DEFAULT_COLOUR

/**
 * Get the border colour for a node type
 */
export const nodeBorderColour = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.borderColour ?? DEFAULT_BORDER_COLOUR
