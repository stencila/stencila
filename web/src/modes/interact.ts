import('./dynamic.ts')

import '../components/nodes/button'
import '../components/nodes/filter'
import '../components/nodes/form'
import '../components/nodes/parameter'

import { waitForElems } from '../utils/curtain'
waitForElems(['button', 'filter', 'form', 'parameter'])
