import type { MessageLevel, NodeType } from '@stencila/types'
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
  // Primitive data types
  Boolean:          { ...shoelaceIcon('toggle-off'),           ...nodeColours('slate')},
  Integer:          { ...shoelaceIcon('hash'),                 ...nodeColours('slate')},
  String:           { ...shoelaceIcon('quote'),                ...nodeColours('slate')},
  Number:           { ...shoelaceIcon('hash'),                 ...nodeColours('slate')},
  Array:            { ...stencilaIcon('array'),                ...nodeColours('slate')},
  Object:           { ...shoelaceIcon('braces'),               ...nodeColours('slate')},
  
  // More complex data and media types
  Datatable:        { ...stencilaIcon('table'),                ...nodeColours('zinc')},
  AudioObject:      { ...shoelaceIcon('volume-up'),            ...nodeColours('zinc'),  title: 'Audio' },
  ImageObject:      { ...shoelaceIcon('image-alt'),            ...nodeColours('zinc'),  title: 'Image' },
  VideoObject:      { ...shoelaceIcon('camera-video'),         ...nodeColours('zinc'),  title: 'Video' },

  // Sections (group blocks so given neutral colour)
  Section:          { ...shoelaceIcon('square'),               ...nodeColours('stone')},

  // Simple block and inline types (usually only have `content` property)
  Heading:          { ...stencilaIcon('heading'),              ...nodeColours('blue')},
  Paragraph:        { ...stencilaIcon('paragraph'),            ...nodeColours('blue')},
  ThematicBreak:    { ...shoelaceIcon('hr'),                   ...nodeColours('blue')},

  // More complex, less common, block and inline types
  Admonition:       { ...stencilaIcon('admonition'),           ...nodeColours('indigo')},
  Claim:            { ...shoelaceIcon('postage'),              ...nodeColours('indigo')},
  List:             { ...stencilaIcon('list'),                 ...nodeColours('indigo')},
  QuoteBlock:       { ...shoelaceIcon('quote'),                ...nodeColours('indigo')},
  Figure:           { ...shoelaceIcon('image'),                ...nodeColours('indigo')},
  Table:            { ...stencilaIcon('table'),                ...nodeColours('indigo')},

  MathBlock:        { ...stencilaIcon('math-block'),           ...nodeColours('cyan')},
  MathInline:       { ...stencilaIcon('math-block'),           ...nodeColours('cyan')},

  // Styled content: use neutral to avoid confusion with styling
  StyledBlock:      { ...shoelaceIcon('palette'),              ...nodeColours('neutral')},
  StyledInline:     { ...shoelaceIcon('palette'),              ...nodeColours('neutral')},
  
  // Static code
  CodeBlock:        { ...shoelaceIcon('braces'),               ...nodeColours('teal')},
  CodeInline:       { ...shoelaceIcon('braces'),               ...nodeColours('teal')},

  // Executable code
  CodeChunk:        { ...stencilaIcon('code-chunk'),           ...nodeColours('green')},
  CodeExpression:   { ...stencilaIcon('code-chunk'),           ...nodeColours('green')},
  
  ForBlock:         { ...stencilaIcon('for-block'),            ...nodeColours('fuchsia')},
  IfBlock:          { ...stencilaIcon('if-block'),             ...nodeColours('pink')},
  
  InstructionBlock: { ...stencilaIcon('instruction-block'),    ...nodeColours('violet')},
  
  InsertBlock:      { ...shoelaceIcon('plus-circle'),          ...nodeColours('lime')},
  ReplaceBlock:     { ...stencilaIcon('replace-block'),        ...nodeColours('orange')},
  DeleteBlock:      { ...shoelaceIcon('dash-circle'),          ...nodeColours('red')}
}

/**
 * Get the UI specifications for a node type
 */
export const nodeUi = (nodeType: NodeType): Required<NodeTypeUI> => {
  const ui = nodeTypeUIMap[nodeType]
  return {
    title: ui?.title ?? nodeType?.replace(/([A-Z])/g, ' $1')?.trim() ?? '',
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

// Execution Messages --------------------------------------

/**
 * Return a colour and an icon for each execution message level
 *
 * @param level The execution level of the message
 * @returns Object containing the twind `colour` string an `icon` name,
 *                 and the icon `library` if not default
 */
export const executionMessageUI = (
  level: MessageLevel
): { colour: string; icon: string; library?: string } => {
  switch (level) {
    case 'Exception':
    case 'Error':
      return { colour: 'pink-900', icon: 'x-circle' }
    case 'Warning':
      return { colour: 'orange-500', icon: 'exclamation-circle' }
    case 'Info':
      return { colour: 'green-900', icon: 'info-circle' }
    case 'Debug':
      return { colour: 'blue-900', icon: 'question-circle' }
    case 'Trace':
      return { colour: 'purple-900', icon: 'slash-circle' }
    default:
      return { colour: 'gray-900', icon: 'circle' }
  }
}

// ---------------------------------------------------------

// Provenance Highlight Colours ----------------------------

export const provenanceHighlights = {
  0: 'transparent',
  1: '#f5f7ff',
  2: '#e4f0ff',
  3: '#d3e3ff',
  4: '#c2d6ff',
  5: '#b1c9fa',
}

export type ProvenanceHighlightLvl = keyof typeof provenanceHighlights

export const getProvHighlight = (miLvl: ProvenanceHighlightLvl) => {
  return provenanceHighlights[miLvl]
}

// ---------------------------------------------------------
