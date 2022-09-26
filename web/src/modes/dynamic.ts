import('./static.ts')

import { DocumentClient } from '../clients/document-client'

const client = new DocumentClient(window.stencilaConfig)
client.connect().catch(console.error)
window.stencilaClient = client
