import {
  EditorPackage as TextureEditorPackage
} from 'substance-texture'

import ReproFigComponent from './ReproFigComponent'
import CellComponent from './CellComponent'
import CodeHighlightComponent from '../shared/CodeHighlightComponent'

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

import {
  SetLanguageCommand, ToggleAllCodeCommand,
  HideCellCodeCommand, CodeErrorsCommand, InsertCellCommand,
  ForceCellOutputCommand
} from './DocumentCommands'

import FunctionUsageCommand from '../shared/FunctionUsageCommand'
import FunctionUsageTool from '../shared/FunctionUsageTool'
import CodeErrorsTool from './CodeErrorsTool'

export default {
  name: 'editor',
  configure(config) {
    config.import(TextureEditorPackage)
    config.addComponent('repro-fig', ReproFigComponent)
    config.addComponent('cell', CellComponent)
    config.addComponent('code-highlight', CodeHighlightComponent)

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

    config.addCommand('insert-cell', InsertCellCommand, {
      nodeType: 'disp-quote',
      commandGroup: 'insert-block-element'
    })
    config.addLabel('insert-cell', 'Cell')
    config.addKeyboardShortcut('CommandOrControl+Enter', { command: 'insert-cell' })

    config.addCommand('function-usage', FunctionUsageCommand, {
      commandGroup: 'prompt'
    })
    config.addTool('function-usage', FunctionUsageTool)

    config.addIcon('function-helper', {'fontawesome': 'fa-question-circle-o' })

    config.addToolPanel('toolbar', [
      {
        name: 'text-types',
        type: 'tool-dropdown',
        showDisabled: false,
        style: 'descriptive',
        commandGroups: ['text-types']
      },
      {
        name: 'persistence',
        type: 'tool-group',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['persistence']
      },
      {
        name: 'annotations',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['formatting']
      },
      {
        name: 'insert',
        type: 'tool-dropdown',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['insert-xref', 'insert-block-element']
      },
      {
        name: 'view',
        type: 'tool-dropdown',
        showDisabled: false,
        style: 'descriptive',
        commandGroups: ['view']
      }
    ])

    config.addToolPanel('node-menu', [
      {
        name: 'cell-actions',
        type: 'tool-group',
        style: 'descriptive',
        showDisabled: false,
        commandGroups: ['cell-actions']
      }
    ])

    /*
      Cell Actions
    */

    // config.addCommand('force-cell-output', ToggleCodeCommand, { forceOutput: true, commandGroup: 'cell-actions' })
    config.addCommand('code-errors', CodeErrorsCommand, {
      commandGroup: 'prompt'
    })
    config.addTool('code-errors', CodeErrorsTool)

    config.addCommand('hide-cell-code', HideCellCodeCommand, { commandGroup: 'cell-actions' })
    config.addCommand('force-cell-output', ForceCellOutputCommand, { commandGroup: 'cell-actions' })
    config.addCommand('set-mini', SetLanguageCommand, { language: 'mini', commandGroup: 'cell-actions' })
    config.addCommand('set-js', SetLanguageCommand, { language: 'js', commandGroup: 'cell-actions' })
    config.addCommand('set-py', SetLanguageCommand, { language: 'py', commandGroup: 'cell-actions' })
    config.addCommand('set-r', SetLanguageCommand, { language: 'r', commandGroup: 'cell-actions' })
    config.addCommand('set-sql', SetLanguageCommand, { language: 'sql', commandGroup: 'cell-actions' })

    // Labels and icons
    config.addLabel('hide-cell-code', 'Hide code')
    config.addLabel('force-cell-output', 'Force output')
    config.addLabel('set-mini', 'Mini')
    config.addLabel('set-js', 'Javascript')
    config.addLabel('set-py', 'Python')
    config.addLabel('set-r', 'R')
    config.addLabel('set-sql', 'SQL')

    config.addIcon('ellipsis', { 'fontawesome': 'fa-ellipsis-v' })
    config.addIcon('test-failed', {'fontawesome': 'fa-times' })
    config.addIcon('test-passed', {'fontawesome': 'fa-check' })

    config.addLabel('view', 'View')
    config.addLabel('show-all-code', 'Show All Code')
    config.addLabel('hide-all-code', 'Hide All Code')

    // View Commands
    config.addCommand('hide-all-code', ToggleAllCodeCommand, {
      hideCode: true,
      commandGroup: 'view'
    })
    config.addCommand('show-all-code', ToggleAllCodeCommand, {
      hideCode: false,
      commandGroup: 'view'
    })

    config.addKeyboardShortcut('CommandOrControl+Alt+L', { command: 'show-all-code' })
    config.addKeyboardShortcut('CommandOrControl+Alt+O', { command: 'hide-all-code' })

  }
}
