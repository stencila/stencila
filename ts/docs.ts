/**
 * Generate Markdown documentation from JSON Schema files.
 *
 * Note that this script requires `public/*.schema.json`
 * and `./types.ts` files. To generate those:
 *
 *     npm run build:jsonschema
 *     npm run build:ts
 */

import * as encoda from '@stencila/encoda'
import { flatten, flow, groupBy, sortBy, startCase, uniq } from 'lodash'
import path from 'path'
import { JsonSchema } from './JsonSchema'
import log from './log'
import {
  Article,
  article,
  BlockContent,
  codeFragment,
  emphasis,
  heading,
  InlineContent,
  Link,
  link,
  list,
  listItem,
  paragraph,
  strong,
  table,
  tableCell,
  tableRow,
} from './types'
import { filterInterfaceSchemas, readSchemas } from './util/helpers'

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (require.main) build()

/**
 * The destination directory for generated `*.md` files
 */
const DOCS_DEST_DIR = path.join(__dirname, '..', 'docs')

/**
 * Generate docs for each `public/*.schema.json` file and
 * convert any `schema/*.md` files to HTML.
 *
 * The generated `public/*.schema.md` file should normally
 * in `include`d into the `schema/*.md` file for the type.
 */
async function build(): Promise<void> {
  log.info('Building docs')

  // Read in all the schemas
  const schemas = filterInterfaceSchemas(await readSchemas())

  // For each schema...
  await Promise.all(
    schemas.map(async (schema) => {
      const { title = '' } = schema
      const summaryArticle = schema2Article(schema)
      await encoda.write(
        summaryArticle,
        path.join(DOCS_DEST_DIR, `${title}.md`)
      )
    })
  )

  // This determines the order in which Schema categories are listed in the Table of Contents
  const orderedCategories = uniq([
    'Prose',
    'Code',
    'Data',
    'Validation',
    'Metadata',
    'Miscellaneous',
    // Any other categories should be listed at the end
    ...Object.values(schemas).map((schema) => startCase(schema.category)),
  ])

  // Group schemas by category, and within each group sort schemas by `status`, and then `name`.
  const groupedSchemas: { [category: string]: JsonSchema[] } = flow([
    (_schemas: JsonSchema[]) =>
      groupBy(_schemas, (schema) =>
        startCase(schema.category ?? 'Miscellaneous')
      ),
    (_schemas: typeof groupedSchemas) =>
      orderedCategories.reduce(
        (categories: typeof groupedSchemas, category) =>
          _schemas[category] !== undefined
            ? {
                ...categories,
                [category]: sortBy(_schemas[category], ['status', 'name']),
              }
            : categories,
        {}
      ),
  ])(schemas)

  const indexPage = article({
    content: Object.entries(groupedSchemas).reduce(
      (prev: BlockContent[], [group, items]) => [
        ...prev,
        heading({
          content: [group],
          depth: 2,
        }),
        list({
          items: items.map((schema) => {
            const { title = 'Untitled' } = schema
            return listItem({
              content: [
                link({
                  content: [startCase(title)],
                  target: `./${title}`,
                }),
              ],
            })
          }),
          order: 'unordered',
        }),
      ],
      []
    ),
  })

  await encoda.write(indexPage, path.join(DOCS_DEST_DIR, `index.md`))
}

/**
 * Given two strings, sort them alphabetically
 */
const sortAlphabetically = (a: string, b: string): number =>
  a < b ? -1 : a > b ? 1 : 0

const requiredPropsFirst = (requiredProps: string[]) => (
  a: string,
  b: string
): number => {
  // If both fields being compared are required, sort alphabetically
  if (requiredProps.includes(a) && requiredProps.includes(b)) {
    return sortAlphabetically(a, b)
  }

  // If field `a` is required and `b` is not, `a` should be listed before `b`
  if (requiredProps.includes(a)) {
    return -1
  }

  // If field `b` is required and `a` is not, `b` should be listed before `a`
  if (requiredProps.includes(b)) {
    return 1
  }

  // If neither fields are required, fall back to sorting them alphabetically
  return sortAlphabetically(a, b)
}

/**
 * Separates elements of an array with a provided separators
 *
 * @template T
 * @template S
 * @param {T[]} array Array of items to separate
 * @param {S} separator value to insert between elements of the array
 * @returns {((T | S)[])}
 */
const intersperse = <T, S>(array: T[], separator: S): (T | S)[] =>
  array
    .reduce((a: (T | S)[], v: T): (T | S)[] => [...a, v, separator], [])
    .slice(0, -1)

/**
 * Formats a sub-schema inside with the correct formatting. Primarily used for
 * encoding allowed types inside an `array` or an `enum`
 *
 * @param {string} prefix
 * @param {Node[]} subTypes
 * @param {string} suffix
 * @returns {Node[]} Stencila Node tree
 */
const subType = (
  prefix: string,
  subTypes: InlineContent[],
  suffix: string
): InlineContent[] => [
  codeFragment({ text: prefix }),
  '​', // These quotes are not empty, they contain a zero-width space character to prevent Markdown decoding errors
  ...subTypes,
  '​', // These quotes are not empty, they contain a zero-width space character to prevent Markdown decoding errors
  codeFragment({ text: suffix }),
]

const orSeparator = ' | '
const andSeparator = ' & '

/**
 * Create a short `InlineContent[]` representation for a JSON Schema.
 * e.g. to be used within a table cell
 *
 * @param {JSON Schema object} content
 * @returns {(InlineContent)[]} An array of InlineContent
 */
const schema2Inlines = (schema: JsonSchema): InlineContent[] => {
  const {
    type = '',
    format,
    items,
    enum: enumeration,
    anyOf,
    allOf,
    $ref,
    parser,
  } = schema

  if (format !== undefined) return [codeFragment({ text: `${type}:${format}` })]
  if (type === 'array' && items !== undefined)
    // Items of an `array` type can either be a single schema object or an
    // array of schemas to validate each item against
    return subType(
      'array[',
      Array.isArray(items)
        ? flatten(items.map(schema2Inlines))
        : schema2Inlines(items),
      ']'
    )
  if (enumeration !== undefined)
    return subType(
      'enum{',
      intersperse(
        enumeration.map((value) => codeFragment({ text: `${value}` })),
        ', '
      ),
      '}'
    )
  if (anyOf !== undefined)
    return flatten(intersperse(anyOf.map(schema2Inlines), orSeparator))
  if (allOf !== undefined)
    return flatten(intersperse(allOf.map(schema2Inlines), andSeparator))
  if ($ref !== undefined) return [ref2Link($ref)]
  if (parser !== undefined) return [codeFragment({ text: `parser:${parser}` })]
  return [codeFragment({ text: `${type}` })]
}

/**
 * Create a summary documentation `Article` for a JSON Schema.
 */
function schema2Article(schema: JsonSchema): Article {
  const {
    title = 'Untitled',
    '@id': id = '',
    description = '',
    properties = {},
    required = [],
    extends: parent,
    descendants = [],
  } = schema

  const tableHeader = tableRow({
    cells: [
      tableCell({ content: ['Name'] }),
      tableCell({ content: ['@id'] }),
      tableCell({ content: ['Type'] }),
      tableCell({ content: ['Description'] }),
      tableCell({ content: ['Inherited from'] }),
    ],
    rowType: 'header',
  })

  const tableData = Object.entries(properties)
    .sort(([a], [b]) => requiredPropsFirst(required)(a, b))
    .map(([name, propSchema]) => {
      const { '@id': id = '', description = '', from = '' } = propSchema
      return tableRow({
        cells: [
          tableCell({
            content: [
              required.includes(name)
                ? strong({
                    content: [name, ' ', emphasis({ content: ['(required)'] })],
                  })
                : name,
            ],
          }),
          tableCell({
            content: [link({ content: [id], target: id2JsonldUrl(id) })],
          }),
          tableCell({ content: schema2Inlines(propSchema) }),
          tableCell({ content: [description] }),
          tableCell({
            content: [link({ content: [from], target: `./${from}` })],
          }),
        ],
      })
    })

  return article({
    content: [
      heading({ content: [title], depth: 1 }),
      paragraph({ content: [description] }),

      heading({ content: ['Properties'], depth: 2 }),
      ...(Object.keys(properties).length > 0
        ? [table({ rows: [tableHeader, ...tableData] })]
        : []),

      heading({ content: ['Related'], depth: 2 }),
      list({
        items: [
          listItem({
            content: [
              paragraph({
                content: [
                  'Parent: ',
                  parent !== undefined ? typeName2Link(parent) : 'None',
                ],
              }),
            ],
          }),
          listItem({
            content: [
              paragraph({
                content: [
                  'Descendants: ',
                  ...(descendants.length !== 0
                    ? intersperse(descendants.map(typeName2Link), ', ')
                    : ['None']),
                ],
              }),
            ],
          }),
        ],
      }),

      paragraph({
        content: [
          ' This documentation was autogenerated from ',
          link({
            content: [codeFragment({ text: `${title}.schema.yaml` })],
            target: `https://github.com/stencila/schema/blob/master/schema/${title}.schema.yaml`,
          }),
          '. This type is also available in ',
          link({
            content: ['JSON-LD'],
            target: id2JsonldUrl(id),
          }),
          ' and ',
          link({
            content: ['JSON Schema'],
            target: id2JsonSchemaUrl(id),
          }),
          '.',
        ],
      }),
    ],
  })
}

/**
 * Generate a link to the Markdown document for a `$ref`
 *
 * @param ref e.g. "Article.schema.json"
 */
const ref2Link = (ref: string): Link => {
  const value = ref.replace('.schema.json', '')
  return link({
    content: [codeFragment({ text: value })],
    target: `./${value}`,
  })
}

/**
 * Generate a link to the Markdown document for a type
 *
 * @param name of the type e.g. "Article"
 */
function typeName2Link(name: string): Link {
  return link({
    content: [name],
    target: `./${name}`,
  })
}

/**
 * Generate a URL for the JSON-LD for an id
 *
 * @param id The id of the type or property, including it's context e.g. `schema:Article`
 */
function id2JsonldUrl(id: string): string {
  const [context, name] = id.split(':')
  return context === 'schema'
    ? `https://schema.org/${name}`
    : `https://schema.stenci.la/${name}.jsonld`
}

/**
 * Generate a URL for the JSON Schema for a type, using it's id
 *
 * @param id The id of the type, including it's context e.g. `schema:Article`
 */
function id2JsonSchemaUrl(id: string): string {
  const [_context, name] = id.split(':')
  return `https://schema.stenci.la/${name}.schema.json`
}
