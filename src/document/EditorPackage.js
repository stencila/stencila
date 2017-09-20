import {
  EditorPackage as TextureEditorPackage
} from 'substance-texture'

import ReproFigComponent from './ReproFigComponent'
import CellComponent from './CellComponent'

export default {
  name: 'editor',
  configure(config) {
    config.import(TextureEditorPackage)
    config.addComponent('repro-fig', ReproFigComponent)
    config.addComponent('cell', CellComponent)
  },
  Editor: TextureEditorPackage.Editor
}
