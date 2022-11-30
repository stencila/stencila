import '../components/editors/prose-editor/prose-editor'
import '../components/nodes/article'
import { waitForElems } from '../utils/curtain'

import('./inspect.ts')
waitForElems(['article', 'stencila-prose-editor'])
