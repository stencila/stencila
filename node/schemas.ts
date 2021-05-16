/**
 * A script to convert JSON Schemas from various Rust modules into TypeScript definitions.
 *
 * To regenerate `src/types.ts` run `npm run build` (this ensures
 * that the `index.node` bindings are up to date first).
 * This is done on CI for each release and `src/types.ts` committed.
 *
 * This file is not in `src` because it is not appropriate for it to end up in `dist`
 * since it relies on `devDependencies`.
 */

import fs from 'fs'
import { compile } from 'json-schema-to-typescript'
import path from 'path'

const addon = require('./index.node')

;(async () => {
  const ts = (
    await Promise.all(
      ['projectsSchema', 'pluginsSchema', 'configSchema'].map(async (name) => {
        const json = addon[name]()
        const schema = JSON.parse(json)
        return compile(schema, name, {
          bannerComment: `/* Generated from '${name}' by '../schemas.ts'. */`,
          style: { semi: false, singleQuote: true },
        })
      })
    )
  ).join('\n')

  fs.writeFileSync(path.join(__dirname, 'src', 'types.ts'), ts)
})()
