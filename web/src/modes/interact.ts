import '../components/nodes/button'
import '../components/nodes/form'
import '../components/nodes/parameter'

import { waitForElems } from '../utils/curtain'

import('./dynamic.ts')
waitForElems(['button', 'form', 'parameter'])
