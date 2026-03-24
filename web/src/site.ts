import { initSiteClient } from './site/client'
import { initSiteGlide } from './site/glide'
import { initUno } from './unocss'

import './views/static'
import './site/components'

initUno()
initSiteClient()
initSiteGlide()
