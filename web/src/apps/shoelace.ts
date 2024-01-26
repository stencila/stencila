import { setBasePath } from '@shoelace-style/shoelace/dist/utilities/base-path.js'

import '@shoelace-style/shoelace/dist/components/icon/icon.js'
import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/menu/menu.js'
import '@shoelace-style/shoelace/dist/components/button/button.js'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'

import { version } from '../../package.json'

const { NODE_ENV } = process.env
const base = NODE_ENV === 'development' ? 'dev' : version

setBasePath(`~static/${base}/shoelace-style/`)
