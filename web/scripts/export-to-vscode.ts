/**
 * A script for exporting code to the VS Code extension
 *
 * Generates files in the `../../vscode/src` directory that should be committed there.
 * Done this way because (a) the extension does not have this package as a dependency,
 * and (b) although we build the `VsCodeView` into the `../../vscode/out` directory, that
 * is for WebViews and can not be imported into the extension code itself.
 */

import { writeFileSync } from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

import { NodeType, NodeTypeList } from '@stencila/types'

import { nodeUi } from '../src/ui/nodes/icons-and-colours'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

// Export colours for each node type

let colours = `// Generated by web/scripts/export-to-vscode.ts at ${new Date().toISOString()}

export const nodeColors: Record<string, string[]> = {
`
for (const nodeType of NodeTypeList) {
  const ui = nodeUi(nodeType as NodeType)
  colours += `  ${nodeType}: ["${ui.colour}", "${ui.borderColour}", "${ui.textColour}"],\n`
}
colours += '}\n'

writeFileSync(
  path.join(__dirname, '..', '..', 'vscode', 'src', 'node-colours.ts'),
  colours
)
