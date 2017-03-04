import { Configurator, BasePackage } from 'substance'
import SheetDocument from './model/SheetDocument'
import SheetDocumentHTMLImporter from './model/SheetDocumentHTMLImporter'
import SheetDocumentHTMLExporter from './model/SheetDocumentHTMLExporter'
import SheetPackage from './SheetPackage'

/**
 * A "configurator" for a sheet.
 *
 * Uses the Substance package mechanism to reduce repetition.
 * See `substance/util/AbstractConfigurator` for inherited methods
 * used by `DocumentHTMLImporter`, `DocumentEditor` etc
 *
 * @class      DocumentConfigurator (name)
 */
class SheetConfigurator extends Configurator {

  constructor () {
    super()

    this.import(BasePackage)
    // At present, need at least the 'default' tool group before adding tools via imports below
    this.addToolGroup('default')

    this.defineSchema({
      name: 'stencila-sheet',
      // FIXME: this does not make sense here
      // as we do not have a container model
      defaultTextType: 'text',
      // FIXME: the name 'ArticleClass' is not general enough
      // plus: the configurator does not fail when this is not specified
      ArticleClass: SheetDocument,
    })

    this.import(SheetPackage)

    this.addImporter('html', SheetDocumentHTMLImporter)
    this.addExporter('html', SheetDocumentHTMLExporter)
  }
}

export default SheetConfigurator
