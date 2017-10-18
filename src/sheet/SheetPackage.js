import { registerSchema, BasePackage } from 'substance'
import SheetDocument from './SheetDocument'
import SheetSchema from './SheetSchema'
import SheetComponent from './SheetComponent'

import {
  InsertRowsAbove, InsertRowsBelow, DeleteRows,
  InsertColumnsLeft, InsertColumnsRight, DeleteColumns,
  OpenColumnSettings, ToggleSheetIssues, SetLanguageCommand,
  SetTypeCommand, EditCellExpressionCommand
} from './SheetCommands'

import EditCellExpressionTool from './EditCellExpressionTool'
import SheetDocumentImporter from './SheetDocumentImporter'
import ColumnSettingsDialog from './ColumnSettingsDialog'
import SheetIssuesComponent from './SheetIssuesComponent'
import SheetIssuesOverlay from './SheetIssuesOverlay'

import BooleanValueComponent from '../shared/BooleanValueComponent'
import NumberValueComponent from '../shared/NumberValueComponent'
import IntegerValueComponent from '../shared/IntegerValueComponent'
import StringValueComponent from '../shared/StringValueComponent'
import ArrayValueComponent from '../shared/ArrayValueComponent'
import ObjectValueComponent from '../shared/ObjectValueComponent'
import TableValueComponent from '../shared/TableValueComponent'
import TestValueComponent from '../shared/TestValueComponent'
import ImageValueComponent from '../shared/ImageValueComponent'
import PlotlyValueComponent from '../shared/PlotlyValueComponent'

import IssueManager from './IssueManager'

export default {
  name: 'Sheet',

  configure(config) {
    // registers model nodes and a converter
    registerSchema(config, SheetSchema, SheetDocument, {
      ImporterClass: SheetDocumentImporter
    })

    config.import(BasePackage)

    config.addManager('issue-manager', IssueManager)

    config.addToolPanel('toolbar', [
      {
        name: 'edit-cell-expression',
        type: 'tool-group',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['edit-cell-expression']
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
      },
      {
        name: 'cell-types',
        type: 'tool-dropdown',
        style: 'descriptive',
        showDisabled: false,
        commandGroups: ['cell-types']
      },
      {
        name: 'cell-languages',
        type: 'tool-dropdown',
        style: 'descriptive',
        showDisabled: false,
        commandGroups: ['cell-languages']
      },
      {
        name: 'undo-redo',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['undo-redo']
      },
    ])

    config.addToolPanel('statusbar', [
      {
        name: 'metrics',
        type: 'tool-group',
        showDisabled: false,
        style: 'minimal',
        commandGroups: ['sheet-issues']
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

    config.addToolPanel('cell-context-menu', [
      {
        name: 'cell-types',
        type: 'tool-group',
        style: 'descriptive',
        showDisabled: true,
        commandGroups: ['cell-types']
      },
      {
        name: 'cell-languages',
        type: 'tool-group',
        style: 'descriptive',
        showDisabled: true,
        commandGroups: ['cell-languages']
      }
    ])

    // Edit Expression Tool
    config.addCommand('edit-cell-expression', EditCellExpressionCommand, { language: 'mini', commandGroup: 'edit-cell-expression' })
    config.addTool('edit-cell-expression', EditCellExpressionTool)

    // Cell Languages
    config.addCommand('set-mini', SetLanguageCommand, { language: undefined, commandGroup: 'cell-languages' })
    config.addCommand('set-js', SetLanguageCommand, { language: 'js', commandGroup: 'cell-languages' })
    config.addCommand('set-py', SetLanguageCommand, { language: 'py', commandGroup: 'cell-languages' })
    config.addCommand('set-r', SetLanguageCommand, { language: 'r', commandGroup: 'cell-languages' })

    config.addLabel('cell-languages', 'Choose Language')
    config.addLabel('set-mini', 'Mini')
    config.addLabel('set-js', 'Javascript')
    config.addLabel('set-py', 'Python')
    config.addLabel('set-r', 'R')

    // Cell Types
    config.addCommand('set-inherit', SetTypeCommand, { type: undefined, commandGroup: 'cell-types' })
    config.addCommand('set-any', SetTypeCommand, { type: 'any', commandGroup: 'cell-types' })
    config.addCommand('set-string', SetTypeCommand, { type: 'string', commandGroup: 'cell-types' })
    config.addCommand('set-number', SetTypeCommand, { type: 'number', commandGroup: 'cell-types' })
    config.addCommand('set-integer', SetTypeCommand, { type: 'integer', commandGroup: 'cell-types' })
    config.addCommand('set-boolean', SetTypeCommand, { type: 'boolean', commandGroup: 'cell-types' })

    config.addLabel('cell-types', 'Choose Cell Type')
    config.addLabel('set-inherit', 'Inherited (${columnType})')
    config.addLabel('set-any', 'Any')
    config.addLabel('set-string', 'String')
    config.addLabel('set-number', 'Number')
    config.addLabel('set-integer', 'Integer')
    config.addLabel('set-boolean', 'Boolean')

    // Labels for types
    config.addLabel('any', 'Any')
    config.addLabel('string', 'String')
    config.addLabel('number', 'Number')
    config.addLabel('integer', 'Integer')
    config.addLabel('boolean', 'Boolean')


    // Cell values
    config.addComponent('value:boolean', BooleanValueComponent)
    config.addComponent('value:integer', IntegerValueComponent)
    config.addComponent('value:number', NumberValueComponent)
    config.addComponent('value:string', StringValueComponent)
    config.addComponent('value:array', ArrayValueComponent)
    config.addComponent('value:object', ObjectValueComponent)
    config.addComponent('value:table', TableValueComponent)
    config.addComponent('value:test', TestValueComponent)
    config.addComponent('value:image', ImageValueComponent)
    config.addComponent('value:plotly', PlotlyValueComponent)


    config.addComponent('sheet', SheetComponent)

    config.addCommand('insert-rows-above', InsertRowsAbove, {
      commandGroup: 'table-row-commands'
    })
    config.addLabel('insert-rows-above', {
      en: 'Insert ${nrows} above'
    })

    config.addCommand('insert-rows-below', InsertRowsBelow, {
      commandGroup: 'table-row-commands'
    })
    config.addLabel('insert-rows-below', {
      en: 'Insert ${nrows} below'
    })
    config.addCommand('delete-rows', DeleteRows, {
      commandGroup: 'table-row-commands'
    })
    config.addLabel('delete-row', {
      en: 'Delete row'
    })
    config.addLabel('delete-rows', {
      en: 'Delete rows ${startRow} - ${endRow}'
    })

    config.addCommand('open-column-settings', OpenColumnSettings, {
      commandGroup: 'table-column-commands'
    })
    config.addLabel('open-column-settings', {
      en: 'Column Settings...'
    })

    config.addCommand('insert-columns-left', InsertColumnsLeft, {
      commandGroup: 'table-column-commands'
    })
    config.addLabel('insert-columns-left', {
      en: 'Insert ${ncolumns} left'
    })

    config.addCommand('insert-columns-right', InsertColumnsRight, {
      commandGroup: 'table-column-commands'
    })
    config.addLabel('insert-columns-right', {
      en: 'Insert ${ncolumns} right'
    })
    config.addCommand('delete-columns', DeleteColumns, {
      commandGroup: 'table-column-commands'
    })
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

    config.addCommand('toggle-sheet-issues', ToggleSheetIssues, {
      commandGroup: 'sheet-issues'
    })
    config.addLabel('toggle-sheet-issues', {
      en: 'Open Issues Panel'
    })
    config.addIcon('sheet-issues', { 'fontawesome': 'fa-warning' })

    config.addComponent('sheet-issues', SheetIssuesComponent)
    config.addComponent('sheet-issues-overlay', SheetIssuesOverlay)

    config.addLabel('title:error', {
      en: 'Error'
    })
    config.addLabel('title:warning', {
      en: 'Warning'
    })

  }
}
