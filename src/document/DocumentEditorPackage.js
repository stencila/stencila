import {
  EditorPackage as TextureEditorPackage
} from 'substance-texture'

import ReproFigComponent from './ReproFigComponent'
import CellComponent from './CellComponent'
import EditExtLinkToolMonkeyPatched from './EditExtLinkToolMonkeyPatched'
import IntegerValueComponent from '../shared/IntegerValueComponent'
import FloatValueComponent from '../shared/FloatValueComponent'
import CodeHighlightComponent from '../shared/CodeHighlightComponent'
import StringValueComponent from '../shared/StringValueComponent'
import ArrayValueComponent from '../shared/ArrayValueComponent'
import TableValueComponent from '../shared/TableValueComponent'
import ImageValueComponent from '../shared/ImageValueComponent'
import ToggleCodeCommand from './ToggleCodeCommand'
import SetLanguageCommand from './SetLanguageCommand'

export default {
  name: 'editor',
  configure(config) {
    config.import(TextureEditorPackage)
    config.addComponent('repro-fig', ReproFigComponent)
    config.addComponent('cell', CellComponent)
    config.addComponent('code-highlight', CodeHighlightComponent)
    config.addComponent('value:integer', IntegerValueComponent)
    config.addComponent('value:float', FloatValueComponent)
    config.addComponent('value:string', StringValueComponent)
    config.addComponent('value:array', ArrayValueComponent)
    config.addComponent('value:table', TableValueComponent)
    config.addComponent('value:image', ImageValueComponent)

    // HACK: override for prototyping FunctionUsage component
    config.addTool('edit-ext-link', EditExtLinkToolMonkeyPatched)

    config.addCommand('hide-cell-code', ToggleCodeCommand, { hideCode: true, commandGroup: 'cell-actions' })
    config.addCommand('show-cell-code', ToggleCodeCommand, { hideCode: false, commandGroup: 'cell-actions' })
    // config.addCommand('force-cell-output', ToggleCodeCommand, { forceOutput: true, commandGroup: 'cell-actions' })
    // config.addCommand('unforce-cell-output', ToggleCodeCommand, { forceOutput: false, commandGroup: 'cell-actions' })
    config.addCommand('set-mini', SetLanguageCommand, { language: 'mini', commandGroup: 'cell-actions' })
    config.addCommand('set-mini', SetLanguageCommand, { language: 'js', commandGroup: 'cell-actions' })
    config.addCommand('set-mini', SetLanguageCommand, { language: 'py', commandGroup: 'cell-actions' })
    config.addCommand('set-mini', SetLanguageCommand, { language: 'r', commandGroup: 'cell-actions' })

    // Labels and icons
    config.addLabel('hide-cell-code', 'Hide code')
    config.addLabel('force-cell-output', 'Show output')
    config.addLabel('set-mini', 'Mini')
    config.addLabel('set-js', 'Javascript')
    config.addLabel('set-py', 'Python')
    config.addLabel('set-r', 'R')

    config.addKeyboardShortcut('CommandOrControl+Alt+H', { command: 'hide-cell-code' })
    config.addKeyboardShortcut('CommandOrControl+Alt+O', { command: 'hide-cell-code' })

    config.addIcon('ellipsis', { 'fontawesome': 'fa-ellipsis-v' })
  }
}
