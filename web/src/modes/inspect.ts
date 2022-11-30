import '../components/nodes/call'
import '../components/nodes/code-chunk'
import '../components/nodes/code-expression'
import '../components/nodes/for'
import '../components/nodes/if'
import '../components/nodes/include'

import { waitForElems } from '../utils/curtain'

import('./interact.ts')
waitForElems(['call', 'code-chunk', 'code-expression', 'for', 'if', 'include'])
