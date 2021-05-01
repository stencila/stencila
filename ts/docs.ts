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
  codeBlock,
  emphasis,
  heading,
  InlineContent,
  Link,
  link,
  list,
  listItem,
  Paragraph,
  paragraph,
  strong,
  table,
  tableCell,
  tableRow,
} from './types'
import { readSchemas } from './util/helpers'

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
 * All the schemas as a map
 */
let SCHEMAS: Record<string, JsonSchema>

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

  // Create a map of schemas, so we can refer to them later
  SCHEMAS = schemas.reduce(
    (prev: Record<string, JsonSchema>, curr: JsonSchema) => {
      return { ...prev, [curr.title ?? 'undef']: curr }
    },
    {}
  )

  // For each schema...
  await Promise.all(
    schemas.map(async (schema) => {
      const { title = '' } = schema
      const summaryArticle = schema2Article(schema)
      await encoda.write(
        summaryArticle,
        path.join(DOCS_DEST_DIR, title2Path(title))
      )
    })
  )

  // This determines the order in which Schema categories are listed in the Table of Contents
  const orderedCategories = uniq([
    'Prose',
    'Code',
    'Data',
    'Media',
    'Other',
    // Any other categories should be listed at the end
    ...Object.values(schemas).map((schema) => startCase(schema.category)),
  ])

  // Group schemas by category, and within each group sort schemas by `name`.
  const groupedSchemas: { [category: string]: JsonSchema[] } = flow([
    (_schemas: JsonSchema[]) =>
      groupBy(_schemas, (schema) => startCase(schema.category ?? 'Other')),
    (_schemas: typeof groupedSchemas) =>
      orderedCategories.reduce(
        (categories: typeof groupedSchemas, category) =>
          _schemas[category] !== undefined
            ? {
                ...categories,
                [category]: sortBy(_schemas[category], ['name']),
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
                  target: title2Path(title),
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

  await encoda.write(indexPage, path.join(DOCS_DEST_DIR, 'Index.md'))
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
    const: const_,
    enum: enumeration,
    anyOf,
    allOf,
    $ref,
    parser,
  } = schema

  if (type === 'array' && items !== undefined) {
    // Items of an `array` type can either be a single schema object or an
    // array of schemas to validate each item against
    const inner = Array.isArray(items)
      ? flatten(items.map(schema2Inlines))
      : schema2Inlines(items)
    return [
      'Array of ',
      inner.length > 1 ? '(' : '',
      ...inner,
      inner.length > 1 ? ')' : '',
    ]
  }
  if (const_ !== undefined) return [`'${const_}'`]
  if (enumeration !== undefined)
    return intersperse(
      enumeration.map((value) => `'${value}'`),
      ', '
    )
  if (anyOf !== undefined)
    return flatten(
      intersperse(anyOf.map(schema2Inlines), [
        ' ',
        emphasis({ content: ['or'] }),
        ' ',
      ])
    )
  if (allOf !== undefined)
    return flatten(
      intersperse(allOf.map(schema2Inlines), [
        ' ',
        emphasis({ content: ['and'] }),
        ' ',
      ])
    )
  if ($ref !== undefined) return [ref2Link($ref)]
  if (parser !== undefined) return [`Parser '${parser}'`]
  if (format !== undefined) return [`Format '${format}'`]
  return [`${type}`]
}

/**
 * Create a summary documentation `Article` for a JSON Schema.
 */
async function schema2Article(schema: JsonSchema): Promise<Article> {
  const {
    title = 'Untitled',
    '@id': id = '',
    anyOf = [],
    description = '',
    properties = {},
    required = [],
    extends: parent,
    descendants = [],
    $comment,
    $id,
    file,
    source,
  } = schema

  // According to https://json-schema.org/draft/2020-12/json-schema-validation.html#rfc.section.9.5
  // this should always be an array
  const examples = schema.examples as Array<unknown>

  const notes: Paragraph[] = []

  if ($comment !== undefined) {
    const para = ((await encoda.load($comment, 'md')) as Article)
      .content?.[0] as Paragraph
    notes.push(para)
  }

  let membersTable
  if (anyOf.length > 0) {
    const tableHeader = tableRow({
      cells: [
        tableCell({ content: ['@id'] }),
        tableCell({ content: ['Type'] }),
        tableCell({ content: ['Description'] }),
      ],
      rowType: 'header',
    })
    const tableData = anyOf.map((memberSchema) => {
      let { $ref, '@id': id, description = '' } = memberSchema
      if ($ref !== undefined) {
        ;({ '@id': id, description = '' } = ref2Schema($ref))
      }
      return tableRow({
        cells: [
          tableCell({ content: id !== undefined ? [id2Link(id)] : [] }),
          tableCell({ content: schema2Inlines(memberSchema) }),
          tableCell({ content: [description] }),
        ],
      })
    })
    membersTable = table({ rows: [tableHeader, ...tableData] })
  }

  let propertiesTable
  if (Object.keys(properties).length > 0) {
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
    const tableData = await Promise.all(
      Object.entries(properties)
        .sort(([a], [b]) => requiredPropsFirst(required)(a, b))
        .map(async ([name, propSchema]) => {
          const {
            '@id': id = '',
            description = '',
            from = '',
            $comment,
          } = propSchema

          let note: InlineContent[] = []
          if ($comment !== undefined) {
            const para = ((await encoda.load($comment, 'md')) as Article)
              .content?.[0] as Paragraph
            notes.push(
              paragraph({
                content: [strong({ content: [name] }), ' : ', ...para?.content],
              })
            )
            note = [
              ' See note ',
              link({ content: [notes.length.toString()], target: '#notes' }),
              '.',
            ]
          }

          return tableRow({
            cells: [
              tableCell({
                content: [
                  required.includes(name) ? strong({ content: [name] }) : name,
                ],
              }),
              tableCell({ content: [id2Link(id)] }),
              tableCell({ content: schema2Inlines(propSchema) }),
              tableCell({ content: [description, ...note] }),
              tableCell({ content: [title2Link(from)] }),
            ],
          })
        })
    )
    propertiesTable = table({ rows: [tableHeader, ...tableData] })
  }

  if (id !== undefined) {
    notes.push(
      paragraph({
        content: [
          'Available as ',
          link({
            content: ['JSON-LD'],
            target: id2JsonldUrl(id),
          }),
          '.',
        ],
      })
    )
  }

  if ($id !== undefined) {
    notes.push(
      paragraph({
        content: [
          'Available as ',
          link({
            content: ['JSON Schema'],
            target: $id,
          }),
          '.',
        ],
      })
    )
  }

  if (file !== undefined && source !== undefined) {
    notes.push(
      paragraph({
        content: [
          'This documentation was generated from ',
          link({
            content: [file],
            target: source,
          }),
          '.',
        ],
      })
    )
  }

  return article({
    // @ts-expect-error Not a property but used for Docusaurus compatibility
    custom_edit_url:
      source !== undefined
        ? source.replace('/blob/', '/edit/')
        : `https://github.com/stencila/schema`,
    content: [
      heading({ content: [title], depth: 1 }),
      paragraph({ content: [description] }),

      ...(membersTable !== undefined
        ? [heading({ content: ['Members'], depth: 2 }), membersTable]
        : []),

      ...(propertiesTable !== undefined
        ? [heading({ content: ['Properties'], depth: 2 }), propertiesTable]
        : []),

      ...(parent !== undefined || descendants.length > 0
        ? [
            heading({ content: ['Related'], depth: 2 }),
            list({
              items: [
                listItem({
                  content: [
                    paragraph({
                      content: [
                        'Parent: ',
                        parent !== undefined ? title2Link(parent) : 'None',
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
                          ? intersperse(descendants.map(title2Link), ', ')
                          : ['None']),
                      ],
                    }),
                  ],
                }),
              ],
            }),
          ]
        : []),

      ...(examples !== undefined && examples.length > 0
        ? [
            heading({ content: ['Examples'], depth: 2 }),
            ...examples.map((example) =>
              codeBlock({
                programmingLanguage: 'json',
                text: JSON.stringify(example, null, '  '),
              })
            ),
          ]
        : []),

      heading({ content: ['Notes'], depth: 2 }),
      list({
        order: 'ascending',
        items: [...notes.map((note) => listItem({ content: [note] }))],
      }),
    ],
  })
}

/**
 * Generate a path to the Markdown document for a schema
 *
 * @param title of the schema e.g. "Article"
 */
function title2Path(title: string, prefix = './'): string {
  const { category = 'Other' } = SCHEMAS[title]
  return `${prefix}${startCase(category)}/${title}.md`
}

/**
 * Generate a link to the Markdown document for a schema
 *
 * @param title of the type e.g. "Article"
 */
function title2Link(title: string): Link {
  return link({
    content: [title],
    target: title2Path(title, '../'),
  })
}

/**
 * Generate a link to the Markdown document for a `$ref`
 *
 * @param ref e.g. "Article.schema.json"
 */
const ref2Link = (ref: string): Link => {
  const title = ref.replace('.schema.json', '')
  return title2Link(title)
}

/**
 * Get the schema from a `$ref`
 *
 * @param ref e.g. "Article.schema.json"
 */
const ref2Schema = (ref: string): JsonSchema => {
  const title = ref.replace('.schema.json', '')
  return SCHEMAS[title]
}

/**
 * Generate a link for an @id, often to an external site
 */
const id2Link = (id: string): Link => {
  const [context, name] = id.split(':')
  const target = (() => {
    switch (context) {
      case 'cito':
        // Appears to be difficult to link to a specific id
        return `https://sparontologies.github.io/cito/current/cito.html`
      case 'schema':
        return `https://schema.org/${name}`
      case 'stencila':
        return id2JsonldUrl(id)
      default:
        return ''
    }
  })()
  return link({ content: [id], target })
}

/**
 * Generate a URL for the JSON-LD for an id
 *
 * @param id The id of the type or property, including it's context e.g. `schema:Article`
 */
function id2JsonldUrl(id: string): string {
  const [_context, name] = id.split(':')
  return `https://schema.stenci.la/${name}.jsonld`
}
