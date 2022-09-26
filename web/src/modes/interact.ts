import('./dynamic.ts')

import '../components/nodes/filter'
import '../components/nodes/form'
import '../components/nodes/gate'
import '../components/nodes/parameter'

import { waitForElems } from '../utils/curtain'
waitForElems(['filter', 'form', 'gate', 'parameter'])
