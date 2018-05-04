import {
  EditorPackage as TextureEditorPackage
} from 'substance-texture'

import ReproFigComponent from './ReproFigComponent'
import ReproFigPreview from './ReproFigPreview'
import CellComponent from './CellComponent'
import CodeHighlightComponent from '../shared/CodeHighlightComponent'

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

import {
  SetLanguageCommand, InsertCellCommand, InsertReproFigCommand,
  ForceCellOutputCommand, RunCellCommand,
  ToggleAllCodeCommand
} from './ArticleEditorCommands'

import FunctionUsageCommand from '../shared/FunctionUsageCommand'
import FunctionUsageTool from '../shared/FunctionUsageTool'
import AutoRunCommand from '../shared/AutoRunCommand'
import RunAllCommand from '../shared/RunAllCommand'

export default {
  name: 'editor',
  configure(config) {
    config.import(TextureEditorPackage)
    config.addComponent('cell', CellComponent)
    config.addComponent('code-highlight', CodeHighlightComponent)

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

    config.addComponent('repro-fig', ReproFigComponent)
    config.addComponent('repro-fig-preview', ReproFigPreview)

    config.addCommand('insert-repro-fig', InsertReproFigCommand, {
      commandGroup: 'insert-repro-figure',
      nodeType: 'repro-fig'
    })
    config.addIcon('insert-repro-fig', { 'fontawesome': 'fa-area-chart' })
    config.addLabel('insert-repro-fig', 'Reproducible Figure')

    config.addCommand('insert-cell', InsertCellCommand, {
      nodeType: 'disp-quote',
      commandGroup: 'insert-cell-element'
    })
    config.addLabel('insert-cell', 'Cell')
    config.addKeyboardShortcut('CommandOrControl+Enter', { command: 'insert-cell' })

    config.addCommand('function-usage', FunctionUsageCommand, {
      commandGroup: 'prompt'
    })
    config.addTool('function-usage', FunctionUsageTool)

    config.addIcon('function-helper', {'fontawesome': 'fa-question-circle' })

    config.addIcon('insert-cell', { 'fontawesome': 'fa-plus-square' })

    config.addLabel('function-examples', {
      en: 'Example Usage'
    })
    config.addLabel('function-usage', {
      en: 'Syntax'
    })

    config.addCommand('auto-run', AutoRunCommand, {
      commandGroup: 'auto-run'
    })

    config.addCommand('run-all', RunAllCommand, {
      commandGroup: 'run-all'
    })
    config.addIcon('run-all', { 'fontawesome': 'fa-caret-square-o-right' })
    config.addLabel('run-all', 'Run All Code')
    config.addKeyboardShortcut('CommandOrControl+Shift+Enter', { command: 'run-all' })

    config.addToolPanel('toolbar', [
      {
        name: 'undo-redo',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['undo-redo']
      },
      {
        name: 'text-types',
        type: 'tool-dropdown',
        showDisabled: false,
        style: 'descriptive',
        commandGroups: ['text-types']
      },
      {
        name: 'annotations',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['formatting']
      },
      {
        name: 'additinal-tools',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['insert-figure', 'insert-repro-figure', 'insert-table', 'insert-cell-element']
      },
      {
        name: 'cell-execution',
        type: 'tool-group',
        showDisabled: false,
        style: 'minimal',
        commandGroups: ['run-all']
      },
      {
        name: 'cite',
        type: 'tool-dropdown',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['insert-xref']
      },
      {
        name: 'view',
        type: 'tool-dropdown',
        showDisabled: false,
        style: 'descriptive',
        commandGroups: ['toggle-content-section', 'view']
      },
      {
        name: 'settings',
        type: 'tool-dropdown',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['auto-run']
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

    config.addCommand(RunCellCommand.name, RunCellCommand, { commandGroup: 'cell-actions' })
    config.addCommand('force-cell-output', ForceCellOutputCommand, { commandGroup: 'cell-actions' })
    config.addCommand('set-mini', SetLanguageCommand, { language: 'mini', commandGroup: 'cell-actions' })
    config.addCommand('set-js', SetLanguageCommand, { language: 'js', commandGroup: 'cell-actions' })
    config.addCommand('set-node', SetLanguageCommand, { language: 'node', commandGroup: 'cell-actions' })
    config.addCommand('set-py', SetLanguageCommand, { language: 'py', commandGroup: 'cell-actions' })
    config.addCommand('set-r', SetLanguageCommand, { language: 'r', commandGroup: 'cell-actions' })
    config.addCommand('set-sql', SetLanguageCommand, { language: 'sql', commandGroup: 'cell-actions' })

    // Labels and icons
    config.addLabel('run-cell-code', 'Run cell')
    //config.addLabel('hide-cell-code', 'Hide code')
    config.addLabel('force-cell-output', 'Force output')
    config.addLabel('set-mini', 'Mini')
    config.addLabel('set-js', 'Javascript')
    config.addLabel('set-node', 'Node.js')
    config.addLabel('set-py', 'Python')
    config.addLabel('set-r', 'R')
    config.addLabel('set-sql', 'SQL')

    config.addIcon('ellipsis', { 'fontawesome': 'fa-ellipsis-v' })
    config.addIcon('test-failed', {'fontawesome': 'fa-times' })
    config.addIcon('test-passed', {'fontawesome': 'fa-check' })


    config.addLabel('show-all-code', 'Show Code')
    config.addLabel('hide-all-code', 'Hide Code')

    config.addLabel('settings', 'Settings')
    config.addLabel('auto-run', '${autoOrManual} Execution')

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
    config.addKeyboardShortcut('Shift+Enter', { command: 'run-cell-code' })

  }
}
