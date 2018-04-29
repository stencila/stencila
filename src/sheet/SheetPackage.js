import { registerSchema, BasePackage } from 'substance'
import SheetDocument from './SheetDocument'
import SheetSchema from './SheetSchema'
import SheetComponent from './SheetComponent'

import {
  InsertRowsAbove, InsertRowsBelow, DeleteRows,
  InsertColumnsLeft, InsertColumnsRight, DeleteColumns,
  OpenColumnSettings, SetLanguageCommand,
  SelectAllCommand
} from './SheetCommands'

import SheetDocumentImporter from './SheetDocumentImporter'
import ColumnSettingsDialog from './ColumnSettingsDialog'

import NullValueComponent from '../shared/NullValueComponent'
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

import CodeHighlightComponent from '../shared/CodeHighlightComponent'

export default {
  name: 'Sheet',

  configure(config) {
    // registers model nodes and a converter
    registerSchema(config, SheetSchema, SheetDocument, {
      ImporterClass: SheetDocumentImporter
    })

    config.addEditorOption({key: 'forcePlainTextPaste', value: true})

    config.import(BasePackage)

    config.addToolPanel('toolbar', [
      {
        name: 'edit-cell-expression',
        type: 'tool-group',
        showDisabled: false,
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
      }
    ])

    config.addToolPanel('statusbar', [
      {
        name: 'metrics',
        type: 'tool-group',
        showDisabled: true,
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
      // TODO: Bring back typed cells
      // {
      //   name: 'cell-types',
      //   type: 'tool-group',
      //   style: 'descriptive',
      //   showDisabled: true,
      //   commandGroups: ['cell-types']
      // },
      {
        name: 'cell-languages',
        type: 'tool-group',
        style: 'descriptive',
        showDisabled: true,
        commandGroups: ['cell-languages']
      }
    ])

    // Cell Languages
    config.addCommand('set-mini', SetLanguageCommand, { language: undefined, commandGroup: 'cell-languages' })
    config.addCommand('set-js', SetLanguageCommand, { language: 'js', commandGroup: 'cell-languages' })
    config.addCommand('set-py', SetLanguageCommand, { language: 'py', commandGroup: 'cell-languages' })
    config.addCommand('set-r', SetLanguageCommand, { language: 'r', commandGroup: 'cell-languages' })
    config.addCommand('set-sql', SetLanguageCommand, { language: 'sql', commandGroup: 'cell-languages' })

    config.addLabel('cell-languages', 'Choose Language')
    config.addLabel('set-mini', 'Mini')
    config.addLabel('set-js', 'Javascript')
    config.addLabel('set-py', 'Python')
    config.addLabel('set-r', 'R')
    config.addLabel('set-sql', 'SQL')

    // TODO: Bring back typed cells
    // // Cell Types
    // config.addCommand('set-inherit', SetTypeCommand, { type: undefined, commandGroup: 'cell-types' })
    // config.addCommand('set-any', SetTypeCommand, { type: 'any', commandGroup: 'cell-types' })
    // config.addCommand('set-string', SetTypeCommand, { type: 'string', commandGroup: 'cell-types' })
    // config.addCommand('set-number', SetTypeCommand, { type: 'number', commandGroup: 'cell-types' })
    // config.addCommand('set-integer', SetTypeCommand, { type: 'integer', commandGroup: 'cell-types' })
    // config.addCommand('set-boolean', SetTypeCommand, { type: 'boolean', commandGroup: 'cell-types' })
    //
    // config.addLabel('cell-types', 'Choose Cell Type')
    // config.addLabel('set-inherit', 'Inherited (${columnType})')
    // config.addLabel('set-any', 'Any')
    // config.addLabel('set-string', 'String')
    // config.addLabel('set-number', 'Number')
    // config.addLabel('set-integer', 'Integer')
    // config.addLabel('set-boolean', 'Boolean')
    //
    // // Labels for types
    // config.addLabel('any', 'Any')
    // config.addLabel('string', 'String')
    // config.addLabel('number', 'Number')
    // config.addLabel('integer', 'Integer')
    // config.addLabel('boolean', 'Boolean')

    // Cell values
    config.addComponent('value:null', NullValueComponent)
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

    config.addComponent('code-highlight', CodeHighlightComponent)

    config.addComponent('sheet', SheetComponent)

    config.addCommand('sheet:select-all', SelectAllCommand)
    config.addKeyboardShortcut('CommandOrControl+a', { command: 'sheet:select-all' })

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

    config.addIcon('toggle-errors', {'fontawesome': 'fa-times-circle' })
    config.addIcon('toggle-warnings', {'fontawesome': 'fa-warning' })
    config.addIcon('toggle-info', {'fontawesome': 'fa-info-circle' })
    config.addIcon('toggle-failed', {'fontawesome': 'fa-times' })
    config.addIcon('toggle-passed', {'fontawesome': 'fa-check' })

    config.addLabel('toggle-errors', 'Errors')
    config.addLabel('toggle-warnings', 'Warnings')
    config.addLabel('toggle-info', 'Info')
    config.addLabel('toggle-failed', 'Test: failed')
    config.addLabel('toggle-passed', 'Test: passed')

    config.addIcon('string-cell-type', {'fontawesome': 'fa-align-left' })
    config.addIcon('number-cell-type', {'fontawesome': 'fa-hashtag' })
    config.addIcon('integer-cell-type', {'fontawesome': 'fa-hashtag' })
    config.addIcon('boolean-cell-type', {'fontawesome': 'fa-check-square-o' })

    config.addLabel('function-reference',' Function Reference')

    config.addLabel('title:error', {
      en: 'Error'
    })
    config.addLabel('title:warning', {
      en: 'Warning'
    })
    config.addLabel('title:info', {
      en: 'Info'
    })

    config.addIcon('test-failed', {'fontawesome': 'fa-times' })
    config.addIcon('test-passed', {'fontawesome': 'fa-check' })

    config.addIcon('context-close', {'fontawesome': 'fa-times' })

    config.addIcon('function-helper', {'fontawesome': 'fa-question-circle' })
    config.addLabel('function-examples', {
      en: 'Example Usage'
    })
    config.addLabel('function-usage', {
      en: 'Syntax'
    })
  }
}
