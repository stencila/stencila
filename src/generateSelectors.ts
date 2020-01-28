import fs from 'fs'
import path from 'path'
import { Types, entityTypes } from '@stencila/schema'
import globby from 'globby'

// Target output path for the file containing generated custom selector definitions
const outputPath = path.join(__dirname, 'common', 'styles', 'selectors.css')

const readSchemas = async (): Promise<string[]> => {
  const paths = path.join(
    path.dirname(require.resolve('@stencila/schema')),
    '*.schema.json'
  )

  return globby([paths]).then(schemaFilePath =>
    schemaFilePath.map(schemaFile => fs.readFileSync(schemaFile).toString())
  )
}

const generateItemProps = async (): Promise<string[]> => {
  const schemas = await readSchemas()

  return schemas
    .reduce(
      (ss: string[], schema) => [
        ...new Set([...ss, ...Object.keys(JSON.parse(schema).properties ?? {})])
      ],
      []
    )
    .sort()
}

const generateItemTypes = (): [string, string][] => {
  const itemTypes: Array<keyof Types> = Object.values(entityTypes).sort()

  const schemaURLs = itemTypes.reduce((types, type) => {
    switch (type) {
      case 'ArrayValidator':
      case 'BooleanValidator':
      case 'CodeBlock':
      case 'CodeChunk':
      case 'CodeError':
      case 'CodeExpression':
      case 'CodeFragment':
      case 'ConstantValidator':
      case 'Datatable':
      case 'DatatableColumn':
      case 'Delete':
      case 'Emphasis':
      case 'Entity':
      case 'EnumValidator':
      case 'Function':
      case 'Include':
      case 'InlineContent':
      case 'IntegerValidator':
      case 'List':
      case 'ListItem':
      case 'Mark':
      case 'Node':
      case 'NumberValidator':
      case 'SoftwareApplication':
      case 'SoftwareSourceCode':
      case 'StringValidator':
      case 'Strong':
      case 'Subscript':
      case 'Superscript':
      case 'ThematicBreak':
      case 'TupleValidator':
      case 'Variable':
        return { ...types, [type]: `https://schema.stenci.la/${type}` }
      default:
        return { ...types, [type]: `https://schema.org/${type}` }
    }
  }, {} as { [key in keyof Types]: string })

  return Object.entries(schemaURLs)
}

const generateSelectors = async (): Promise<void> => {
  let selectors = ''

  generateItemTypes().map(
    ([type, schemaURL]) =>
      (selectors += `@custom-selector :--${type} [itemtype='${schemaURL}'];\n`)
  )

  let props = ''

  const itemProps = await generateItemProps()

  itemProps.map(p => {
    props += `@custom-selector :--${p} [itemprop='${p}'];\n`
  })

  const doc = `/**
*
* THIS FILE IS AUTO-GENERATED. DO NOT MODIFY MANUALLY.
*
* Custom, semantic, CSS selectors
*
* In Thema the approach to CSS naming is, as much as possible, to
* use custom selectors for the type of entities being styled and
* their properties.
*/

/**
 * To tweak or adjust an existing theme, you may override some common CSS variables found in the themes.

 * Available CSS variables are:
 * --color-accent: Color for accent elements, primarily used to add a brand highlights to the theme.
 * --color-key: Color for body text, and other elements using the inherited body text color.
 * --color-neutral: Subtle color, usually shades of gray, for element borders and other subtle details.
 * --color-stock: Article/Page background color, but also used for other elements.
 * --font-family-body: Font-family for paragraphs and other non-headline elements
 * --font-family-heading: Font-family for paragraphs and other non-headline elements
 * --font-family-mono: Font-family for monospaced text elements such as \`pre\` and \`code\`.
 * --max-width-media: Maximum width for media content, including images and interactive Code Chunks.
 * --max-width: Max width for textual elements and other non-media content.
 *
 * Note that not all themes make use of all available variables, and that some may expose additional options.
 * Please refer to the specific theme documentation.
 */

/**
 * Type selectors
 *
 * For types defined in http://schema.org (e.g. \`Article\`), or extensions such as,
 * http://schema.stenci.la (e.g. \`CodeChunk\`), http://bioschemas.org (e.g. \`Taxon\`) etc.
 *
 * Conventions:
 *
 * - use the same upper camel case as in the schema the type is defined in
 * - use a \`[itemtype=...]\` selector if possible (i.e. if Encoda encodes it in HTML)
 */
${selectors}

/**
 * Property selectors
 *
 * For properties of types defined in schemas. Note that
 * some of these select an entire container property e.g. \`authors\` and
 * selector for a class, and some select items in those properties
 * e.g. \`author\` and select for a \`itemprop\`.
 *
 * Conventions:
 *
 * - use the same lower camel case as in the schema the property is defined in
 * - use a \`.class\` selector for container properties
 * - use a \`[itemprop=...]\` selector for singular properties, or items of container properties
 */
${props}
  `

  return fs.writeFile(outputPath, doc, () => doc)
}

// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) generateSelectors()
