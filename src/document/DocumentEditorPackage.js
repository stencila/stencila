import {
  EditorPackage as TextureEditorPackage
} from 'substance-texture'

import ReproFigComponent from './ReproFigComponent'
import CellComponent from './CellComponent'
import IntegerValueComponent from '../shared/IntegerValueComponent'
import FloatValueComponent from '../shared/FloatValueComponent'
import CodeHighlightComponent from '../shared/CodeHighlightComponent'
import StringValueComponent from '../shared/StringValueComponent'
import ArrayValueComponent from '../shared/ArrayValueComponent'
import TableValueComponent from '../shared/TableValueComponent'
import ImageValueComponent from '../shared/ImageValueComponent'

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
  }
}
