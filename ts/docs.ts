/**
 * Generate documentation
 */

/* eslint-disable @typescript-eslint/strict-boolean-expressions */

import * as encoda from '@stencila/encoda'
import encodaProcess from '@stencila/encoda/dist/process'
import fs from 'fs-extra'
import globby from 'globby'
import { flatten } from 'lodash'
import path from 'path'
// The main reason this is imported is to configure the log handling
import log from './log'
import { Article, Code, code, Link, link, Node, Strong } from './types'
import { isArticle } from './util/guards'
import Schema from './schema.d'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
docs()

/**
 * Generate docs for each `built/*.schema.json` file and
 * convert any `schema/*.md` files to HTML.
 *
 * The generated `built/*.schema.md` file should normally
 * in `include`d into the `schema/*.md` file for the type.
 */
async function docs(): Promise<void> {
  log.info('Building docs')

  const schemas = await globby('built/*.schema.json')

  await Promise.all(
    schemas.map(async jsonFile => {
      try {
        const schema = await fs.readJSON(jsonFile)
        const { title } = schema

        const articleMd = schema2Article(schema)
        const schemaMdFile = path.join('built', `${title}.schema.md`)
        await encoda.write(articleMd, schemaMdFile)

        const mdFile = path.join('schema', `${title}.md`)
        if (await fs.pathExists(mdFile)) {
          const article = await encoda.read(mdFile)

          // Don't output an HTML file if the Markdown document cannot be decoded into an article
          if (!isArticle(article)) {
            return
          }

          const processed = await encodaProcess(
            {
              ...article,
              title,
              content: [
                {
                  type: 'Heading',
                  depth: 2,
                  content: [
                    'Status: ',
                    schema.status,
                    ', ',
                    'Role: ',
                    schema.role
                  ]
                },
                ...(article.content || [])
              ]
            },
            path.dirname(mdFile)
          )

          const htmlFile = path.join('built', `${title}.html`)
          await encoda.write(processed, htmlFile, {
            isBundle: false, // Set isBundle to true to work locally with NPM linked Thema style changes
            theme: 'stencila'
          })
        }
      } catch (error) {
        console.error(error)
      }
    })
  )

  // Cover over any files generated during processing
  // so that links in HTML files work
  const outs = await globby('schema/*.out.*')
  await Promise.all(
    outs.map(async file => {
      return fs.copy(file, path.join('built', path.basename(file)))
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
 * @param {T[]} arr Array of items to separate
 * @param {S} separator value to insert between elements of the array
 * @returns {((T | S)[])}
 */
const intersperse = <T, S extends T>(arr: T[], separator: S): (T | S)[] =>
  arr.reduce((a: T[], v: T): (T | S)[] => [...a, v, separator], []).slice(0, -1)

const linkifyReference = (ref: string): Link => {
  const value = ref.replace('.schema.json', '')
  return link([code(value)], `./${value}.html`)
}

const formattedType = (type: string, format: string): Code =>
  code(`${type}<${format}>`)

/**
 * Formats a sub-schema inside with the correct formatting. Primarily used for
 * encoding allowed types inside an `array` or an `enum`
 *
 * @param {string} prefix
 * @param {Node[]} subTypes
 * @param {string} suffix
 * @returns {Node[]} Stencila Node tree
 */
const subType = (prefix: string, subTypes: Node[], suffix: string): Node[] => [
  code(prefix),
  '​', // These quotes are not empty, they contain a zero-width space character to prevent Markdown decoding errors
  ...subTypes,
  '​', // These quotes are not empty, they contain a zero-width space character to prevent Markdown decoding errors
  code(suffix)
]

const orSeparator = ' | '
const andSeparator = ' & '

/**
 * Parse and convert JSON Schema to a Stencila Node tree representation to be rendered for the documentation website
 *
 * @param {JSON Schema object} content
 * @returns {(Node)[]} Stencila Node tree
 */
const encodeContents = (content: any): (Node)[] => {
  if (typeof content.format === 'string') {
    return [formattedType(content.type, content.format)]
  }

  if (content.type === 'array' && content.items) {
    // Items of an `array` type can either be a single schema object or an array of schemas to validate each item against
    const items: (Node)[] = Array.isArray(content.items)
      ? flatten(content.items.map(encodeContents))
      : encodeContents(content.items)

    return subType('array<', items, '>')
  }

  if (content.enum) {
    return subType(
      'enum<',
      intersperse<Node, string>(content.enum.map(code), orSeparator),
      '>'
    )
  }

  if (content.anyOf) {
    return flatten(intersperse(content.anyOf.map(encodeContents), orSeparator))
  }

  if (content.allOf) {
    return flatten(intersperse(content.allOf.map(encodeContents), andSeparator))
  }

  if (content.$ref) {
    return [linkifyReference(content.$ref)]
  }

  if (content.codec) {
    return [code(`codec<${content.codec}>`)]
  }

  return [code(content.type)]
}

/**
 * Create an article from a JSON schema object using
 * properties like `description`, `parent` etc.
 */
function schema2Article(schema: { [key: string]: any }): Article {
  const { title = 'Untitled', properties = {} } = schema

  const requiredProps = schema.required || []

  // Differentiate required properties by bolding them and adding a `(required)` suffix
  const requiredWrapper = (name: string): Strong | string =>
    requiredProps.includes(name)
      ? {
          type: 'Strong',
          content: [name, ' ', { type: 'Emphasis', content: ['(required)'] }]
        }
      : name

  const propertiesTable = {
    type: 'Table',
    rows: Object.entries(properties)
      .sort(([a], [b]) => requiredPropsFirst(requiredProps)(a, b))
      .map(([name, prop]: [string, any]) => {
        const { description = '', from = '' } = prop
        return {
          type: 'TableRow',
          cells: [
            {
              type: 'TableCell',
              content: [requiredWrapper(name)]
            },
            {
              type: 'TableCell',
              content: encodeContents(prop)
            },
            {
              type: 'TableCell',
              content: [description]
            },
            {
              type: 'TableCell',
              content: [
                {
                  type: 'Link',
                  target: `./${from}.html`,
                  content: [from]
                }
              ]
            }
          ]
        }
      })
  }

  const article: Article = {
    type: 'Article',
    title,
    authors: [],
    content: [
      {
        type: 'Paragraph',
        content: [schema.description || '']
      },
      {
        type: 'Heading',
        depth: 2,
        content: ['Properties']
      },
      propertiesTable
    ]
  }

  return article
}
