/**
 * A script to update the `json.schemas` property of the `./vscode/settings.json` file.
 *
 * This enabled intellisense for files following the naming conversion of `.<type>.json`.
 * For example, `a.Cite.json` file will have `Cite.schema.json` applied to it within
 * the editor.
 *
 * @see https://code.visualstudio.com/docs/languages/json#_json-schemas-and-settings
 *
 * Run using `npx ts-node ts/bindings/vscode.ts`
 */

import fs from 'fs-extra'
import path from 'path'
import { filterInterfaceSchemas, readSchemas } from '../util/helpers'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
;(async () => {
  const mappings = filterInterfaceSchemas(await readSchemas()).map(
    ({ title }) => ({
      fileMatch: [`*.${title ?? ''}.json`],
      url: `./public/${title ?? ''}.schema.json`,
    })
  )
  const file = path.join(__dirname, '..', '..', '.vscode', 'settings.json')
  fs.writeJsonSync(
    file,
    { ...fs.readJsonSync(file), 'json.schemas': mappings },
    { spaces: 2 }
  )
})()
