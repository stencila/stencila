/**
 * Script for rendering a Stencila JSON document to static HTML
 *
 * Requires `rollup.conf.js` etc ala examples in
 * https://github.com/solidjs/solid/tree/main/packages/solid-ssr
 */

import * as schema from '@stencila/schema'
import fs from 'fs'
import { renderToString } from 'solid-js/web'
import { MetaProvider, renderTags } from 'solid-meta'
import { DocumentRenderer } from './Document'
import { ThemeRenderer } from './Theme'

/**
 * Encode a `CreativeWork` to HTML with a theme
 */
export function encode(document: schema.CreativeWork, theme?: string): string {
  const tags: any[] = []
  const content = renderToString(() => (
    <MetaProvider tags={tags}>
      <ThemeRenderer theme={theme}>
        <DocumentRenderer document={document}></DocumentRenderer>
      </ThemeRenderer>
    </MetaProvider>
  ))
  return `
  <!doctype html>
    <head>
      ${renderTags(tags)}
    </head>
    <body>
      <div id="root">${content}</div>
    </body>
  </html>
`
}

/**
 * Simple CLI interface to be able to run this standalone for testing.
 */
export function cli() {
  const from = process.argv[3]
  const to = process.argv[4]
  const theme = process.argv[5]
  console.info(`Rendering "${from}" to "${to}" with theme "${theme}"`)

  const json = fs.readFileSync(from, 'utf-8')
  const document = JSON.parse(json) as schema.CreativeWork
  const html = encode(document, theme)
  fs.writeFileSync(to, html)
}

if (require.main === module) {
  cli()
}
