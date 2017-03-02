import SheetModel from './model/SheetModel'
import Cell from './model/Cell'
import CellHTMLConverter from './model/CellHTMLConverter'
import SheetHTMLImporter from './model/SheetHTMLImporter'
import SheetHTMLExporter from './model/SheetHTMLExporter'

import CellComponent from './ui/CellComponent'
import BooleanComponent from './ui/BooleanComponent'
import ConstantComponent from './ui/ConstantComponent'
import PrimitiveComponent from './ui/PrimitiveComponent'
import ExpressionComponent from './ui/ExpressionComponent'
import ErrorComponent from './ui/ErrorComponent'
import HTMLCellComponent from './ui/HTMLCellComponent'
import ImageCellComponent from './ui/ImageCellComponent'

export default {
  name: 'sheet',
  configure: function(config, options = {}) {
    config.defineSchema({
      name: 'stencila-sheet',
      // FIXME: this does not make sense here
      // as we do not have a container model
      defaultTextType: 'text',
      // FIXME: the name 'ArticleClass' is not general enough
      // plus: the configurator does not fail when this is not specified
      ArticleClass: SheetModel,
    })
    config.addNode(Cell)
    config.addConverter('html', CellHTMLConverter)
    config.addImporter('html', SheetHTMLImporter)
    config.addExporter('html', SheetHTMLExporter)

    config.addComponent('cell', CellComponent)
    config.addComponent('cell:boolean', BooleanComponent)
    config.addComponent('cell:constant', ConstantComponent)
    config.addComponent('cell:primitive', PrimitiveComponent)
    config.addComponent('cell:expression', ExpressionComponent)
    config.addComponent('cell:error', ErrorComponent)
    config.addComponent('cell:html', HTMLCellComponent)
    config.addComponent('cell:image', ImageCellComponent)
  }
}