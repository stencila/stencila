import {
  type AdmonitionType,
  type MessageLevel,
  type NodeType,
} from '@stencila/types'
import tailwindConfig from 'tailwindcss/defaultConfig'
import resolveConfig from 'tailwindcss/resolveConfig'

import { IconName } from '../icons/icon'

export type NodeTypeUI = {
  title?: string
  icon?: IconName
  colour?: string
  borderColour?: string
  textColour?: string
}

const colours = resolveConfig(tailwindConfig).theme.colors

const DEFAULT_ICON = 'box'
const DEFAULT_COLOUR = colours.blue[100]
const DEFAULT_BORDER_COLOUR = colours.blue[200]
const DEFAULT_TEXT_COLOUR = colours.blue[900]

const nodeColours = (name: string) => ({
  colour: colours[name][100],
  borderColour: colours[name][200],
  textColour: colours[name][900],
})

// prettier-ignore
const nodeTypeUIMap: Partial<Record<NodeType, NodeTypeUI>> = {
  // Article level
  Article:          {                        ...nodeColours('gray')},


  // Primitive data types
  Boolean:          { icon: 'toggleOff',     ...nodeColours('slate')},
  Integer:          { icon: 'hash',          ...nodeColours('slate')},
  String:           { icon: 'quote',         ...nodeColours('slate')},
  Number:           { icon: 'hash',          ...nodeColours('slate')},
  Array:            { icon: 'array',         ...nodeColours('slate')},
  Object:           { icon: 'braces',        ...nodeColours('slate')},
  
  // More complex data and media types
  Datatable:        { icon: 'table',         ...nodeColours('zinc')},
  AudioObject:      { icon: 'volumeUp',      ...nodeColours('zinc'),  title: 'Audio' },
  ImageObject:      { icon: 'imageAlt',      ...nodeColours('zinc'),  title: 'Image' },
  VideoObject:      { icon: 'cameraVideo',   ...nodeColours('zinc'),  title: 'Video' },

  // Sections (group blocks so given neutral colour)
  Section:          { icon: 'square',         ...nodeColours('stone')},

  // Simple block and inline types (usually only have `content` property)
  Heading:          { icon: 'heading',        ...nodeColours('blue')},
  Paragraph:        { icon: 'paragraph',      ...nodeColours('blue')},
  ThematicBreak:    { icon: 'hr',             ...nodeColours('blue')},

  // More complex, less common, block and inline types
  Admonition:       { icon: 'admonition',     ...nodeColours('indigo')},
  Claim:            { icon: 'postage',        ...nodeColours('indigo')},
  List:             { icon: 'list',           ...nodeColours('indigo')},
  QuoteBlock:       { icon: 'quote',          ...nodeColours('indigo')},
  Figure:           { icon: 'image',          ...nodeColours('indigo')},
  Table:            { icon: 'table',          ...nodeColours('indigo')},

  MathBlock:        { icon: 'mathBlock',      ...nodeColours('cyan')},
  MathInline:       { icon: 'mathBlock',      ...nodeColours('cyan')},

  // Styled content: use neutral to avoid confusion with styling
  StyledBlock:      { icon: 'brush',        ...nodeColours('neutral')},
  StyledInline:     { icon: 'brush',        ...nodeColours('neutral')},
  
  // Static code
  CodeBlock:        { icon: 'braces',         ...nodeColours('teal')},
  CodeInline:       { icon: 'braces',         ...nodeColours('teal')},

  // Executable code
  CodeChunk:        { icon: 'codeChunk',      ...nodeColours('green')},
  CodeExpression:   { icon: 'codeChunk',      ...nodeColours('green')},
  
  ForBlock:         { icon: 'repeat',         ...nodeColours('fuchsia')},
  IfBlock:          { icon: 'ifBlock',        ...nodeColours('pink')},

  IncludeBlock:     { icon: 'filePlus',       ...nodeColours('sky')},
  CallBlock:        { icon: 'filePlay',       ...nodeColours('lime')},
  
  InstructionBlock:   { icon: 'chatRightDots', ...nodeColours('violet')},
  InstructionMessage: { icon: 'chatRightText', ...nodeColours('violet')},
  SuggestionBlock:    { icon: 'cardText',      ...nodeColours('indigo'),  title: 'Suggestion' },
  PromptBlock:        { icon: 'chatRightText', ...nodeColours('purple'),  title: 'Prompt Preview'},
  
  InsertBlock:      { icon: 'plusCircle',         ...nodeColours('lime')},
  ReplaceBlock:     { icon: 'replaceBlock',       ...nodeColours('orange')},
  DeleteBlock:      { icon: 'dashCircle',         ...nodeColours('red')},
}

/**
 * Get the UI specifications for a node type
 */
export const nodeUi = (nodeType: NodeType): Required<NodeTypeUI> => {
  const ui = nodeTypeUIMap[nodeType]
  return {
    title: ui?.title ?? nodeType?.replace(/([A-Z])/g, ' $1')?.trim() ?? '',
    icon: ui?.icon ?? DEFAULT_ICON,
    colour: ui?.colour ?? DEFAULT_COLOUR,
    borderColour: ui?.borderColour ?? DEFAULT_BORDER_COLOUR,
    textColour: ui?.textColour ?? DEFAULT_TEXT_COLOUR,
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
): { colour: string; icon: IconName } => {
  switch (level) {
    case 'Exception':
    case 'Error':
      return { colour: 'pink-900', icon: 'xCircle' }
    case 'Warning':
      return { colour: 'orange-500', icon: 'exclamationTriangle' }
    case 'Info':
      return { colour: 'green-900', icon: 'infoCircle' }
    case 'Debug':
      return { colour: 'blue-900', icon: 'questionCircle' }
    case 'Trace':
      return { colour: 'purple-900', icon: 'slashCircle' }
    default:
      return { colour: 'gray-900', icon: 'circle' }
  }
}

// ---------------------------------------------------------

// Provenance Highlight Colours ----------------------------

const provenanceOpacity = {
  0: '1',
  1: '0.9',
  2: '0.8',
  3: '0.7',
  4: '0.6',
  5: '0.5',
}

export type ProvenanceOpacityLevel = keyof typeof provenanceOpacity

export const getProvenanceOpacity = (level: ProvenanceOpacityLevel) => {
  return provenanceOpacity[level]
}

// ---------------------------------------------------------

// Admonition UI -------------------------------------------

type AdmonitionTypeUI = {
  baseColour: string
  borderColour: string
  textColour: string
  icon: IconName
}

const admonitionColours = (name: string) => ({
  baseColour: colours[name][50],
  borderColour: colours[name][400],
  textColour: colours[name][800],
})

// prettier-ignore
const admonitionUiMap: Record<AdmonitionType, AdmonitionTypeUI> = {
  Note:      { ...admonitionColours('blue'),     icon:'infoCircle' },
  Info:      { ...admonitionColours('blue'),     icon:'infoCircle' },
  Tip:       { ...admonitionColours('green'),    icon:'lightbulb' },
  Important: { ...admonitionColours('blue'),     icon:'exclamationCircle' },
  Success:   { ...admonitionColours('green'),    icon:'checkCircle' },
  Failure:   { ...admonitionColours('red'),      icon:'xCircle' },
  Warning:   { ...admonitionColours('yellow'),   icon:'exclamationTriangle' },
  Danger:    { ...admonitionColours('red'),      icon:'exclamationCircle' },
  Error:     { ...admonitionColours('red'),      icon:'xCircle' },
}

export const admonitionUi = (type: AdmonitionType) => {
  return admonitionUiMap[type]
}

// ---------------------------------------------------------
