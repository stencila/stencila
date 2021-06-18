/**
 * Generate Markdown documentation from JSON Schema files.
 *
 * Note that this script requires `public/*.schema.json`
 * and `./types.ts` files. To generate those:
 *
 *     npm run build:jsonschema
 *     npm run build:ts
 */

/* eslint-disable @typescript-eslint/restrict-template-expressions */

import fs from 'fs-extra'
import { flatten, flow, groupBy, sortBy, startCase, uniq } from 'lodash'
import path from 'path'
import { JsonSchema } from './JsonSchema'
import log from './log'
import { readSchemas } from './util/helpers'
import yaml from 'js-yaml'

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
 * Generate docs for each `public/*.schema.json` file.
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

  // Generate articles for each schema
  await Promise.all(
    schemas.map(async (schema) => {
      const { title = '' } = schema
      const summaryArticle = schema2Article(schema)
      await fs.writeFile(
        path.join(DOCS_DEST_DIR, title2Path(title)),
        summaryArticle
      )
    })
  )

  // This determines the order in which Schema categories are listed in the
  // index.md & categories.json files
  const orderedCategories = uniq([
    'Works',
    'Text',
    'Math',
    'Code',
    'Data',
    'Other',
    // Any other categories should be listed at the end
    ...Object.values(schemas).map((schema) => startCase(schema.category)),
  ])

  // Group schemas by category, and within each group sort schemas by `name`.
  const groupedSchemas = flow([
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
  ])(schemas) as { [category: string]: JsonSchema[] }

  // Generate index.md
  const indexPage = article({
    title: 'Index',
    content: [
      paragraph([
        'All the schemas in ',
        link('Stencila Schema', 'https://schema.stenci.la'),
        ' by category.',
      ]),
      ...Object.entries(groupedSchemas).reduce(
        (prev: string[], [group, items]) => [
          ...prev,
          heading(group, 2),
          list(
            items.map((schema) => {
              const { title = 'Untitled' } = schema
              return link(startCase(title), title2Path(title))
            }),
            'Unordered'
          ),
        ],
        []
      ),
    ],
  })
  await fs.writeFile(path.join(DOCS_DEST_DIR, 'index.md'), indexPage)

  // Generate categories.json for use by Docusaurus
  const categories = Object.entries(groupedSchemas).map(
    ([category, schemas]) => {
      return {
        type: 'category',
        label: category,
        items: schemas.map((schema) => `schema/docs/${schema.title}`),
      }
    }
  )
  await fs.writeJSON(path.join(DOCS_DEST_DIR, 'categories.json'), categories, {
    spaces: 2,
  })
}

/**
 * Given two strings, sort them alphabetically
 */
const sortAlphabetically = (a: string, b: string): number =>
  a < b ? -1 : a > b ? 1 : 0

const requiredPropsFirst =
  (requiredProps: string[]) =>
  (a: string, b: string): number => {
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
const schema2Inlines = (schema: JsonSchema): string[] => {
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
      intersperse(anyOf.map(schema2Inlines), [' ', emphasis('or'), ' '])
    )
  if (allOf !== undefined)
    return flatten(
      intersperse(allOf.map(schema2Inlines), [' ', emphasis('and'), ' '])
    )
  if ($ref !== undefined) return [ref2Link($ref)]
  if (parser !== undefined) return [`Parser '${parser}'`]
  if (format !== undefined) return [`Format '${format}'`]
  return [`${type}`]
}

/**
 * Create a documentation article for a JSON Schema.
 */
function schema2Article(schema: JsonSchema): string {
  const {
    category = 'Other',
    title = 'Untitled',
    '@id': id = '',
    anyOf = [],
    status = 'experimental',
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

  const hasProperties = Object.keys(properties).length > 0

  const commentContent =
    $comment !== undefined
      ? paragraph([$comment.replace(/\n/g, ' ')])
      : !id.startsWith('stencila:') && hasProperties
      ? paragraph(['This type is an implementation of ', id2Link(id), '.'])
      : ''

  const statusNote =
    status !== 'stable'
      ? [
          paragraph([
            'This schema type is marked as ',
            strong(status),
            status === 'experimental' ? ' 🧪' : ' ⚠️',
            ' and is subject to change.',
          ]),
        ]
      : []

  let membersTable
  if (anyOf.length > 0) {
    const tableHeader = tableRow(['@id', 'Type', 'Description'])
    const tableData = anyOf.map((memberSchema) => {
      let { $ref, '@id': id, description = '' } = memberSchema
      if ($ref !== undefined) {
        ;({ '@id': id, description = '' } = ref2Schema($ref))
      }
      return tableRow([
        id !== undefined ? id2Link(id) : '',
        schema2Inlines(memberSchema).join(''),
        description,
      ])
    })
    membersTable = table([tableHeader, ...tableData])
  }

  const notes: string[] = []
  let propertiesTable
  if (hasProperties) {
    const tableHeader = tableRow([
      'Name',
      codeFragment('@id'),
      'Type',
      'Description',
      'Inherited from',
    ])
    const tableData = Object.entries(properties)
      // Don't show the `type` property, it's not interesting
      .filter(([name, _propSchema]) => name !== 'type')
      .sort(([a], [b]) => requiredPropsFirst(required)(a, b))
      .map(([name, propSchema]) => {
        const {
          '@id': id = '',
          description = '',
          from = '',
          $comment,
        } = propSchema

        let note: string[] = []
        if ($comment !== undefined) {
          notes.push(
            paragraph([strong(name), ' : ', $comment.replace(/\n/g, ' ')])
          )
          note = [' See note ', link(notes.length.toString(), '#notes'), '.']
        }

        return tableRow([
          required.includes(name) ? strong(name) : name,
          id2Link(id),
          schema2Inlines(propSchema).join(''),
          [description.replace(/\n/g, ' '), ...note].join(''),
          title2Link(from),
        ])
      })
    propertiesTable = table([tableHeader, ...tableData])
  }

  const availableAs = []
  if (id !== undefined) {
    availableAs.push(paragraph([link('JSON-LD', id2JsonldUrl(id))]))
  }
  if ($id !== undefined) {
    availableAs.push(paragraph([link('JSON Schema', $id)]))
  }
  if (id !== undefined && hasProperties) {
    availableAs.push(
      paragraph([
        'Python ',
        link(codeFragment(`class ${title}`), id2PythonDocs(id)),
      ]),
      paragraph([
        'TypeScript ',
        link(codeFragment(`interface ${title}`), id2TypeScriptDocs(id)),
      ]),
      paragraph(['R ', link(codeFragment(`class ${title}`), id2RDocs(id))]),
      paragraph([
        'Rust ',
        link(codeFragment(`struct ${title}`), id2RustDocs(id)),
      ])
    )
  }

  return article({
    category: startCase(category),
    slug: `/schema/${title}`,
    custom_edit_url:
      source !== undefined
        ? source.replace('/blob/', '/edit/')
        : `https://github.com/stencila/schema`,
    // Main article content
    content: [
      heading(startCase(title), 1),
      paragraph([strong(description)]),

      ...commentContent,

      ...statusNote,

      ...(membersTable !== undefined
        ? [heading('Members', 2), membersTable]
        : []),

      ...(propertiesTable !== undefined
        ? [heading('Properties', 2), propertiesTable]
        : []),

      ...(notes.length > 0
        ? [
            heading('Notes', 2),
            list(
              notes.map((note) => listItem([note])),
              'Ascending'
            ),
          ]
        : []),

      ...(examples !== undefined && examples.length > 0
        ? [
            heading('Examples', 2),
            ...examples.map((example) =>
              codeBlock({
                programmingLanguage: 'json',
                text: JSON.stringify(example, null, '  '),
              })
            ),
          ]
        : []),

      ...(parent !== undefined || descendants.length > 0
        ? [
            heading('Related', 2),
            list([
              listItem([
                paragraph([
                  'Parent: ',
                  parent !== undefined ? title2Link(parent) : 'None',
                ]),
              ]),
              listItem([
                paragraph([
                  'Descendants: ',
                  ...(descendants.length !== 0
                    ? intersperse(descendants.map(title2Link), ', ')
                    : ['None']),
                ]),
              ]),
            ]),
          ]
        : []),

      ...(availableAs.length > 0
        ? [
            heading('Available as', 2),
            list(availableAs.map((item) => listItem([item]))),
          ]
        : []),

      ...(file !== undefined && source !== undefined
        ? [
            heading('Source', 2),
            paragraph([
              'This documentation was generated from ',
              link(file, source),
              '.',
            ]),
          ]
        : []),
    ],
  })
}

// Functions for generating Markdown, same naming and similar
// args (in some cases) as Schema constructor functions but generating
// Markdown directly instead of using Encoda (which causes a somewhat
// unwieldy circular dependency, albeit a dev dependency)

function article(node: { content: string[]; [key: string]: unknown }): string {
  const { content, ...rest } = node
  return `---\n${yaml.dump(rest).trim()}\n---\n\n${content.join('')}\n`
}

function heading(content: string, depth: number): string {
  return `${Array(depth).fill('#').join('')} ${content}\n\n`
}

function paragraph(content: string[]): string {
  return `${content.join('')}\n\n`
}

function table(rows: string[]): string {
  const [first, ...rest] = rows
  return `${first}\n${first.replace(/[^|]/g, '-')}\n${rest.join('\n')}\n\n`
}

function tableRow(cells: string[]): string {
  return cells.join(' | ')
}

function list(items: string[], ordering = 'Unordered'): string {
  return ordering === 'Unordered'
    ? `${items.map((item) => `- ${item.trim()}\n`).join('')}\n`
    : `${items
        .map((item, index) => `${index + 1}. ${item.trim()}\n`)
        .join('')}\n`
}

function listItem(content: string[]): string {
  return content.join('')
}

function codeBlock(node: {
  text: string
  programmingLanguage: string
}): string {
  const { programmingLanguage, text } = node
  return `\`\`\`${programmingLanguage}\n${text}\n\`\`\`\n\n`
}

function emphasis(content: string): string {
  return `_${content}_`
}

function strong(content: string): string {
  return `**${content}**`
}

function codeFragment(text: string): string {
  return `\`${text}\``
}

function link(content: string, target: string): string {
  return `[${content}](${target})`
}

/**
 * Generate a path to the Markdown document for a schema
 *
 * @param title of the schema e.g. "Article"
 */
function title2Path(title: string): string {
  return `${title}.md`
}

/**
 * Generate a link to the Markdown document for a schema
 *
 * @param title of the type e.g. "Article"
 */
function title2Link(title: string): string {
  return link(title, title2Path(title))
}

/**
 * Generate a link to the Markdown document for a `$ref`
 *
 * @param ref e.g. "Article.schema.json"
 */
const ref2Link = (ref: string): string => {
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
const id2Link = (id: string): string => {
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
  return link(id, target)
}

/**
 * Generate a URL for the JSON-LD for an id
 *
 * @param id The id of the type or property, including it's context e.g. `schema:Article`
 */
function id2JsonldUrl(id: string): string {
  const [_context, name = 'stencila'] = id.split(':')
  return `https://schema.stenci.la/${name}.jsonld`
}

/**
 * Generate a URL for the Python docs for an id
 *
 * @param id The id of the type, including it's context e.g. `schema:Article`
 */
function id2PythonDocs(id: string): string {
  const [_context, name = ''] = id.split(':')
  return `https://stencila.github.io/schema/python/docs/types.html#schema.types.${name}`
}

/**
 * Generate a URL for the TypeScript docs for an id
 *
 * @param id The id of the type, including it's context e.g. `schema:Article`
 */
function id2TypeScriptDocs(id: string): string {
  const [_context, name = ''] = id.split(':')
  return `https://stencila.github.io/schema/ts/docs/interfaces/${name.toLowerCase()}.html`
}

/**
 * Generate a URL for the Python docs for an id
 *
 * @param id The id of the type, including it's context e.g. `schema:Article`
 */
function id2RDocs(_id: string): string {
  return `https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf`
}

/**
 * Generate a URL for the Rust docs for an id
 *
 * @param id The id of the type, including it's context e.g. `schema:Article`
 */
function id2RustDocs(id: string): string {
  const [_context, name = ''] = id.split(':')
  return `https://docs.rs/stencila-schema/latest/stencila_schema/struct.${name}.html`
}
