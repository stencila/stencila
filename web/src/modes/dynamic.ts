import('./static.ts')

import { DocumentClient } from '../clients/document-client'
import { waitForElems } from '../utils/curtain'

import '../components/nodes/division'
import '../components/nodes/math-block'
import '../components/nodes/math-fragment'
import '../components/nodes/span'

// Use DocumentClient to provide dynamically updating content via
// received patches
const client = new DocumentClient(window.stencilaConfig)
client.connect().catch(console.error)
window.stencilaClient = client

// Uses Web Components for `Styled` and `Math` nodes to provide dynamically
// updating CSS and MathML respectively for those node types (patching alone will not do that)
waitForElems(['division', 'span', 'math-block', 'math-fragment'])
