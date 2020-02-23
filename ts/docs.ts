/**
 * Generate documentation
 *
 * Note that this script requires `public/*.schema.json`
 * and `./types.ts` files. To generate those:
 *
 *     npm run build:jsonschema
 *     npm run build:ts
 */

import * as encoda from '@stencila/encoda'
import fs from 'fs-extra'
import globby from 'globby'
import { flow, groupBy, sortBy, startCase, uniq } from 'lodash'
import flatten from 'lodash.flatten'
import path from 'path'
import { readSchemas } from './helpers'
import log from './log'
import JsonSchema from './jsonSchema'
import {
  Article,
  article,
  codeFragment,
  emphasis,
  heading,
  InlineContent,
  Link,
  link,
  list,
  ListItem,
  listItem,
  paragraph,
  strong,
  superscript,
  table,
  tableCell,
  tableRow
} from './types'

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()

/**
 * The source directory for docs e.g. `*.md` files
 */
const DOCS_SOURCE_DIR = path.join(__dirname, '..', 'schema')

/**
 * The destination directory for docs e.g. `*.html` and `*.md` files
 */
const DOCS_DEST_DIR = path.join(__dirname, '..', 'public')

/**
 * String marker appended to Schema links in the documentation to indicate
 * Unstable status, and that the underlying schema may change at any time.
 */
const unstableMarker = superscript({ content: ['U'] })

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
  const schemas = await readSchemas()

  // For each schema...
  await Promise.all(
    schemas.map(async schema => {
      const { title } = schema

      // 1. Generate a summary article for the schema and write it
      // to disk so that it can be transcluded in manually written
      // Markdown files if desired.
      const summaryArticle = schema2SummaryArticle(schema)
      await encoda.write(
        summaryArticle,
        path.join(DOCS_DEST_DIR, `${title}.schema.md`)
      )

      // 2. If the schema has a manually main Article then use
      // that, otherwise generate it
      // TODO: Reinstate this when manually written docs can be
      // better converted by Encoda
      /*
      const mdFile = path.join(DOCS_SOURCE_DIR, `${title}.md`)
      const mainArticle = (await fs.pathExists(mdFile))
        ? await encoda.read(mdFile)
        : schema2MainArticle(schema, summaryArticle)
      */
      const mainArticle = schema2MainArticle(schema, summaryArticle)

      // 3. Write the main article as HTML
      await encoda.write(mainArticle, path.join(DOCS_DEST_DIR, `${title}.html`))
    })
  )

  // Copy over any output files that may have been generated
  // when running `encoda compile` on the main Article so that
  // links to those files in HTML files are not broken
  const outputs = await globby('schema/*.out.*')
  await Promise.all(
    outputs.map(async file => {
      return fs.copy(file, path.join(DOCS_DEST_DIR, path.basename(file)))
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
    ...Object.values(schemas).map(schema => startCase(schema.category))
  ])

  // Group schemas by category, and within each group sort schemas by `status`, and then `name`.
  const groupedSchemas: { [category: string]: JsonSchema[] } = flow([
    (_schemas: JsonSchema[]) =>
      groupBy(_schemas, schema =>
        startCase(schema.category ?? 'Miscellaneous')
      ),
    (_schemas: typeof groupedSchemas) =>
      orderedCategories.reduce(
        (categories: typeof groupedSchemas, category) =>
          _schemas[category] !== undefined
            ? {
                ...categories,
                [category]: sortBy(_schemas[category], ['status', 'name'])
              }
            : categories,
        {}
      )
  ])(schemas)

  // Generate the index page list of links
  const schemaList = list({
    items: Object.entries(groupedSchemas).map(([group, items]) =>
      listItem({
        content: [
          strong({
            content: [group]
          }),
          list({
            items: items.map(schema2ListItem),
            order: 'unordered'
          })
        ]
      })
    ),
    order: 'unordered'
  })

  const readme = (await encoda.read(
    path.join(DOCS_SOURCE_DIR, '..', 'README.md')
  )) as Article
  const readmeContent = readme.content !== undefined ? readme.content : []

  const indexPage = article({
    authors: [],
    title: 'Stencila Schema',
    content: [
      ...readmeContent,
      heading({ content: ['Available types'], depth: 2 }),
      paragraph({
        content: [
          emphasis({
            content: [
              'Schemas marked with “',
              unstableMarker,
              '” are considered unstable and have a higher likelihood of changes.'
            ]
          })
        ]
      }),
      schemaList
    ]
  })

  await encoda.write(indexPage, path.join(DOCS_DEST_DIR, `index.html`))

  // Convert other documentation to HTML
  const others = await globby('docs/*')
  await Promise.all(
    others.map(async file => {
      const { name } = path.parse(file)
      return encoda.convert(file, path.join(DOCS_DEST_DIR, `${name}.html`))
    })
  )
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

const linkifyReference = (ref: string): Link => {
  const value = ref.replace('.schema.json', '')
  return link({
    content: [codeFragment({ text: value })],
    target: `./${value}.html`
  })
}

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
  codeFragment({ text: suffix })
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
    codec
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
        enumeration.map(value => codeFragment({ text: `${value}` })),
        ', '
      ),
      '}'
    )
  if (anyOf !== undefined)
    return intersperse(flatten(anyOf.map(schema2Inlines)), orSeparator)
  if (allOf !== undefined)
    return intersperse(flatten(allOf.map(schema2Inlines)), andSeparator)
  if ($ref !== undefined) return [linkifyReference($ref)]
  if (codec !== undefined) return [codeFragment({ text: `codec:${codec}` })]
  return [codeFragment({ text: `${type}` })]
}

/**
 * Create a  main documentation `Article` for a JSON Schema.
 *
 * Ideally these articles will be written manually, but this functions provides
 * for the cases in which it has not.
 *
 * As well as the generated summary we could automatically insert examples here as
 * @100ideas did in https://github.com/stencila/schema/pull/142
 */
function schema2MainArticle(
  schema: JsonSchema,
  summaryArticle?: Article
): Article {
  const { title = 'Untitled' } = schema
  const { content: summary = [] } =
    summaryArticle !== undefined
      ? summaryArticle
      : schema2SummaryArticle(schema)

  return article({
    title,
    authors: [],
    content: [
      ...summary,
      paragraph({
        content: [
          strong({ content: ['Note:'] }),
          ' This documentation was autogenerated from ',
          link({
            content: [codeFragment({ text: `${title}.schema.yaml` })],
            target: `https://github.com/stencila/schema/blob/master/schema/${title}.schema.yaml`
          }),
          '. ',
          'Please help improve these docs (and show how we humans 💁 can do better than bots 🤖!) by ',
          link({
            content: ['creating a Markdown documentation file'],
            target:
              'https://github.com/stencila/schema/blob/master/docs/writing-schema-docs.md'
          }),
          '🙏'
        ]
      })
    ]
  })
}

/**
 * Create a summary documentation `Article` for a JSON Schema.
 */
function schema2SummaryArticle(schema: JsonSchema): Article {
  const {
    title = 'Untitled',
    '@id': id = '',
    description = '',
    properties = {},
    required = [],
    extends: parent,
    descendants = []
  } = schema

  const tableHeader = tableRow({
    cells: [
      tableCell({ content: ['Name'] }),
      tableCell({ content: ['@id'] }),
      tableCell({ content: ['Type'] }),
      tableCell({ content: ['Description'] }),
      tableCell({ content: ['Inherited from'] })
    ],
    rowType: 'header'
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
                    content: [name, ' ', emphasis({ content: ['(required)'] })]
                  })
                : name
            ]
          }),
          tableCell({ content: [propertyId2Link(id)] }),
          tableCell({ content: schema2Inlines(propSchema) }),
          tableCell({ content: [description] }),
          tableCell({
            content: [link({ content: [from], target: `./${from}.html` })]
          })
        ]
      })
    })

  return article({
    authors: [],
    title,
    content: [
      paragraph({ content: [description] }),
      paragraph({
        content: [strong({ content: ['@id'] }), ': ', typeId2Link(id)]
      }),
      parent !== undefined
        ? paragraph({
            content: [
              strong({ content: ['Parent'] }),
              ': ',
              typeName2Link(parent)
            ]
          })
        : '',
      descendants.length !== 0
        ? paragraph({
            content: [
              strong({ content: ['Descendants'] }),
              ': ',
              ...intersperse(descendants.map(typeName2Link), ', ')
            ]
          })
        : '',
      ...(Object.keys(properties).length > 0
        ? [
            heading({ content: ['Properties'], depth: 2 }),
            table({ rows: [tableHeader, ...tableData] })
          ]
        : [])
    ]
  })
}

function typeId2Link(id: string): Link {
  const [context, name] = id.split(':')
  const target =
    context === 'schema'
      ? `https://schema.org/${name}`
      : `https://schema.stenci.la/jsonld/${name}`
  return link({
    content: [id],
    target
  })
}

function typeName2Link(name: string): Link {
  return link({
    content: [name],
    target: `./${name}.html`
  })
}

function propertyId2Link(id: string): Link {
  const [context, name] = id.split(':')
  const target =
    context === 'schema'
      ? `https://schema.org/${name}`
      : `https://schema.stenci.la/jsonld/${name}`
  return link({
    content: [id],
    target
  })
}

/**
 * Create a list item for a schema to be used in a list of links.
 */
function schema2ListItem(schema: JsonSchema): ListItem {
  const { title = 'Untitled' } = schema
  const status = schema.status === 'unstable' ? [unstableMarker] : []

  return listItem({
    content: [
      link({
        content: [startCase(title), ...status],
        target: `${title}.html`
      })
    ]
  })
}
