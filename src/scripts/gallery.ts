/**
 * A script to generate `../docs/gallery.html`
 *
 * Run using `npm run docs:gallery`. Noting that this
 * uses built themes in `docs/themes` so `npm run docs:app`
 * needs to be done first.
 */

import { read, write, dump, shutdown } from '@stencila/encoda'
import {
  Article,
  article,
  CreativeWork,
  creativeWork,
  imageObject,
  link,
  list,
  listItem,
  organization
} from '@stencila/schema'
import { tmpdir } from 'os'
import path from 'path'
import { themes } from '../themes'

const themesDir = path.join(__dirname, '..', 'themes')
const examplesDir = path.join(__dirname, '..', 'examples')
const docsDir = path.join(__dirname, '..', '..', 'docs')

const stencila = organization({
  name: 'Stencila',
  logo: 'https://stenci.la/img/logo-name.png'
})

if (module.parent === null)
  generateGallery().catch(error => {
    console.log(error)
    process.exit(1)
  })

/**
 * Generate the "gallery", an `Article` with a `List` of `CreativeWork` nodes,
 * one for each theme.
 *
 * Also outputs a JSON file that can be used by the demo page to provide for
 * information about a theme.
 */
async function generateGallery(): Promise<void> {
  const example = await read(path.join(examplesDir, 'articleKitchenSink.json'))

  const summaries = (
    await Promise.all(
      Object.keys(themes).map(
        async (theme): Promise<[string, CreativeWork]> => [
          theme,
          await generateSummary(theme, `?theme=${theme}`, example as Article)
        ]
      )
    )
  ).reduce(
    (prev: Record<string, CreativeWork>, [key, value]) => ({
      ...prev,
      [key]: value
    }),
    {}
  )

  await write(summaries, path.join(docsDir, 'themes.json'))

  const gallery = article({
    title: 'Thema Gallery',
    authors: [stencila],
    publisher: stencila,
    datePublished: new Date().toISOString(),
    content: [
      list({
        items: Object.entries(summaries).map(([theme, summary]) => {
          return listItem({
            // TODO: change this from `meta.url` to `url` after refactoring `ListItem` schema
            meta: { url: `?theme=${theme}` },
            // TODO: change this from `content` to `item` after refactoring `ListItem` schema
            content: [summary]
          })
        })
      })
    ]
  })

  await write(gallery, path.join(docsDir, 'gallery.html'), {
    isStandalone: true,
    theme: path.join(docsDir, 'themes', 'galleria')
  })

  await shutdown()
}

/**
 * Generate a `CreativeWork` for each theme based on it's README.md
 * file and adding a screenshot.
 */
async function generateSummary(
  theme: string,
  url: string,
  example: Article
): Promise<CreativeWork> {
  // Read the README and use defaults for undefined properties
  const {
    authors = [],
    publisher = stencila,
    description,
    content = [],
    ...rest
  } = (await read(path.join(themesDir, theme, 'README.md'))) as CreativeWork

  // Generate a screenshot using the local build of the theme
  const screenshot = path.join(tmpdir(), 'screenshots', `${theme}.png`)
  await write(example, screenshot, {
    theme: path.join(docsDir, 'themes', theme),
    size: { height: 500, width: 800 }
  })

  // Make the creative work
  return creativeWork({
    ...rest,
    publisher,
    // New content includes the screenshot
    content: [
      ...content,
      link({
        target: url,
        content: [imageObject({ contentUrl: screenshot })]
      })
    ],
    // If their is not a description in the YAML meta data of the
    // README, then make it the plain text version of the original content
    description:
      description !== undefined ? description : await dump(content, 'txt'),
    // HTML version of the original content
    text: await dump(content, 'html')
  })
}
