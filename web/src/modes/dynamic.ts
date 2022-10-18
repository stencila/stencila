import('./static.ts')

import { DocumentClient } from '../clients/document-client'
import { waitForElems } from '../utils/curtain'

import '../components/nodes/division'
import '../components/nodes/span'

// Use DocumentClient to provide dynamically updating content via
// received patches
const client = new DocumentClient(window.stencilaConfig)
client.connect().catch(console.error)
window.stencilaClient = client

// Uses Web Components for Division and Span nodes to provide dynamically
// updating CSS for those node types (patching alone will not do that)
waitForElems(['division', 'span'])
