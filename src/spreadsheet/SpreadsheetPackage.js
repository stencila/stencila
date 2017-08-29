import { registerSchema, BasePackage } from 'substance'
import SpreadsheetDocument from './SpreadsheetDocument'
import SpreadsheetSchema from './SpreadsheetSchema'

import {
  InsertRowsAbove, InsertRowsBelow,
  InsertColumnsLeft, InsertColumnsRight
} from './SpreadsheetCommands'
import {
  InsertRowsAboveTool, InsertRowsBelowTool,
  InsertColumnsLeftTool, InsertColumnsRightTool,
 } from './SpreadsheetTools'
 import SpreadsheetDocumentImporter from './SpreadsheetDocumentImporter'

export default {
  name: 'Spreadsheet',

  configure(config) {
    // registers model nodes and a converter
    registerSchema(config, SpreadsheetSchema, SpreadsheetDocument, {
      ImporterClass: SpreadsheetDocumentImporter
    })

    config.import(BasePackage)

    config.addToolPanel('toolbar', [
      {
        name: 'undo-redo',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['undo-redo']
      },
      {
        name: 'annotations',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['annotations']
      }
    ])

    config.addToolPanel('row-context-menu', [
      {
        name: 'row-context-menu',
        type: 'tool-group',
        style: 'descriptive',
        showDisabled: true,
        commandGroups: ['table-row-commands']
      }
    ])

    config.addToolPanel('column-context-menu', [
      {
        name: 'column-context-menu',
        type: 'tool-group',
        style: 'descriptive',
        showDisabled: true,
        commandGroups: ['table-column-commands']
      }
    ])

    config.addCommand('insert-rows-above', InsertRowsAbove, {
      commandGroup: 'table-row-commands'
    })
    config.addTool('insert-rows-above', InsertRowsAboveTool)
    config.addLabel('insert-rows-above', {
      en: 'Insert ${n} above'
    })

    config.addCommand('insert-rows-below', InsertRowsBelow, {
      commandGroup: 'table-row-commands'
    })
    config.addTool('insert-rows-below', InsertRowsBelowTool)
    config.addLabel('insert-rows-below', {
      en: 'Insert ${n} below'
    })

    config.addCommand('insert-columns-left', InsertColumnsLeft, {
      commandGroup: 'table-column-commands'
    })
    config.addTool('insert-columns-left', InsertColumnsLeftTool)
    config.addLabel('insert-columns-left', {
      en: 'Insert ${n} left'
    })

    config.addCommand('insert-columns-right', InsertColumnsRight, {
      commandGroup: 'table-column-commands'
    })
    config.addTool('insert-columns-right', InsertColumnsRightTool)
    config.addLabel('insert-columns-right', {
      en: 'Insert ${n} right'
    })

  }
}
