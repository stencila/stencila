import {
  EditorPackage as TextureEditorPackage
} from 'substance-texture'

import ReproFigComponent from './ReproFigComponent'
import CellComponent from './CellComponent'
import EditExtLinkToolMonkeyPatched from './EditExtLinkToolMonkeyPatched'
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

    // HACK: override

    config.addTool('edit-ext-link', EditExtLinkToolMonkeyPatched)
  }
}
