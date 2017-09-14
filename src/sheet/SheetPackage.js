import { registerSchema, BasePackage } from 'substance'
import SheetDocument from './SheetDocument'
import SheetSchema from './SheetSchema'

import {
  InsertRowsAbove, InsertRowsBelow, DeleteRows,
  InsertColumnsLeft, InsertColumnsRight, DeleteColumns,
  OpenColumnSettings, OpenSheetIssues
} from './SheetCommands'
import {
  InsertRowsAboveTool, InsertRowsBelowTool, DeleteRowsTool,
  InsertColumnsLeftTool, InsertColumnsRightTool, DeleteColumnsTool,
  OpenColumnSettingsTool, OpenSheetIssuesTool
 } from './SheetTools'

import SheetDocumentImporter from './SheetDocumentImporter'
import ColumnSettingsDialog from './ColumnSettingsDialog'

export default {
  name: 'Sheet',

  configure(config) {
    // registers model nodes and a converter
    registerSchema(config, SheetSchema, SheetDocument, {
      ImporterClass: SheetDocumentImporter
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
      },
      {
        name: 'sheet-inspector',
        type: 'tool-group',
        showDisabled: false,
        style: 'minimal',
        commandGroups: ['sheet-inspector']
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
    config.addCommand('delete-rows', DeleteRows, {
      commandGroup: 'table-row-commands'
    })
    config.addTool('delete-rows', DeleteRowsTool)
    config.addLabel('delete-row', {
      en: 'Delete row'
    })
    config.addLabel('delete-rows', {
      en: 'Delete rows ${startRow} - ${endRow}'
    })

    config.addCommand('open-column-settings', OpenColumnSettings, {
      commandGroup: 'table-column-commands'
    })
    config.addTool('open-column-settings', OpenColumnSettingsTool)
    config.addLabel('open-column-settings', {
      en: 'Column Settings...'
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
    config.addCommand('delete-columns', DeleteColumns, {
      commandGroup: 'table-column-commands'
    })
    config.addTool('delete-columns', DeleteColumnsTool)
    config.addLabel('delete-column', {
      en: 'Delete column'
    })
    config.addLabel('delete-columns', {
      en: 'Delete columns ${startCol} - ${endCol}'
    })

    config.addIcon('sheet-scroll-left', { 'fontawesome': 'fa-angle-left' })
    config.addIcon('sheet-scroll-right', { 'fontawesome': 'fa-angle-right' })
    config.addIcon('sheet-scroll-up', { 'fontawesome': 'fa-angle-up' })
    config.addIcon('sheet-scroll-down', { 'fontawesome': 'fa-angle-down' })

    config.addComponent('column-settings-dialog', ColumnSettingsDialog)
    config.addLabel('title:column-settings', {
      en: 'Column Settings'
    })

    config.addCommand('open-sheet-issues', OpenSheetIssues, {
      commandGroup: 'sheet-inspector'
    })
    config.addTool('open-sheet-issues', OpenSheetIssuesTool)
    config.addLabel('open-sheet-issues', {
      en: 'Open Issues'
    })
    config.addIcon('open-sheet-issues', { 'fontawesome': 'fa-warning' })

    // config.addCommand('open-sheet-inspector', OpenSheetInspector, {
    //   commandGroup: 'sheet-inspector'
    // })
    // config.addTool('open-sheet-inspector', OpenSheetInspectorTool)
    // config.addLabel('open-sheet-inspector', {
    //   en: 'Open Sheet-Inspector'
    // })
    // config.addIcon('open-sheet-inspector', { 'fontawesome': 'fa-wrench' })
  }
}
