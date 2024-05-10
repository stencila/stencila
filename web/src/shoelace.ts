/**
 * Configuration of Shoelace
 */

// TODO: These imports would be better to be done where needed
import '@shoelace-style/shoelace/dist/components/button/button.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'
import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/icon-button/icon-button.js'
import '@shoelace-style/shoelace/dist/components/icon/icon.js'
import '@shoelace-style/shoelace/dist/components/input/input.js'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js'
import '@shoelace-style/shoelace/dist/components/menu/menu.js'
import '@shoelace-style/shoelace/dist/components/tree-item/tree-item.js'
import '@shoelace-style/shoelace/dist/components/tree/tree.js'

// Import Shoelace related CSS so it is included in any bundle that
// itself imports this file
import '@shoelace-style/shoelace/dist/themes/light.css'
import '@shoelace-style/shoelace/dist/themes/dark.css'
import './shoelace.css'

import { setBasePath } from '@shoelace-style/shoelace/dist/utilities/base-path.js'
import { registerIconLibrary } from '@shoelace-style/shoelace/dist/utilities/icon-library.js'

import { version } from '../package.json'

const { NODE_ENV, STENCILA_VIEW } = process.env

// For builds for the VSCode extension it is necessary to use a placeholder
// that the extension can dynamically replace with a fully qualified WebView URL
// which includes the extension filesystem path
const base =
  STENCILA_VIEW === 'vscode'
    ? 'VSCODE_BASE_URL'
    : NODE_ENV === 'development'
      ? '/~static/dev'
      : `/~static/${version}`

setBasePath(`${base}/shoelace/`)

registerIconLibrary('stencila', {
  resolver: (name) => `${base}/stencila/assets/icons/${name}.svg`,
})

registerIconLibrary('boxicons', {
  resolver: (name) => {
    let folder = 'regular'
    if (name.substring(0, 4) === 'bxs-') folder = 'solid'
    if (name.substring(0, 4) === 'bxl-') folder = 'logos'
    return `https://cdn.jsdelivr.net/npm/boxicons@2.1.4/svg/${folder}/${name}.svg`
  },
  mutator: (svg) => svg.setAttribute('fill', 'currentColor'),
})

registerIconLibrary('lucide', {
  resolver: (name) =>
    `https://cdn.jsdelivr.net/npm/lucide-static@0.365.0/icons/${name}.svg`,
})

export type ShoelaceIconLibraries =
  | 'default'
  | 'stencila'
  | 'boxicons'
  | 'lucide'
