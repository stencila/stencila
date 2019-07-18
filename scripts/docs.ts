/**
 * Generate documentation
 */

import * as encoda from '@stencila/encoda'
import encodaProcess from '@stencila/encoda/dist/process'
import fs from 'fs-extra'
import globby from 'globby'
import path from 'path'

docs()

/**
 * Generate docs for each `built/*.schema.json` file and
 * convert any `schema/*.md` files to HTML.
 *
 * The generated `built/*.schema.md` file should normally
 * in `include`d into the `schema/*.md` file for the type.
 */
async function docs() {
  const schemas = await globby('built/*.schema.json')

  await Promise.all(schemas.map(async jsonFile => {
    const schema = await fs.readJSON(jsonFile)
    const { title } = schema

    const article = schema2Article(schema)
    const schemaMdFile = path.join('built', `${title}.schema.md`)
    await encoda.write(article, schemaMdFile)

    const mdFile = path.join('schema', `${title}.md`)
    if (await fs.pathExists(mdFile)) {
      const article = await encoda.read(mdFile)
      const processed = await encodaProcess(article, path.dirname(mdFile))

      const htmlFile = path.join('built', `${title}.html`)
      await encoda.write(processed, htmlFile)
    }
  }))
}

/**
 * Create an article from a JSON schema object using
 * properties like `description`, `parent` etc.
 */
function schema2Article(schema: {[key: string]: any}) {
  const { title = 'Untitled', properties = {} } = schema

  const propertiesTable = {
    type: 'Table',
    rows: Object.entries(properties)
      // TODO: Maybe sort properties in ascending order of inheritance depth
      // and then alphabetically, like schema.org
      .map(([name, prop]: [string, any]) => {
        const { description = '', type = '', from = '' } = prop
        return {
          type: 'TableRow',
          cells: [
            {
              type: 'TableCell',
              content: [from]
            },
            {
              type: 'TableCell',
              content: [name]
            },
            {
              type: 'TableCell',
              content: [description]
            },
            {
              type: 'TableCell',
              content: [type]
            }
          ]
        }
      })
  }

  const article = {
    type: 'Article',
    title,
    authors: [],
    content: [
      {
        type: 'Paragraph',
        content: [schema.description || '']
      },
      propertiesTable
    ]
  }

  return article
}
