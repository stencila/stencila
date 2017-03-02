import { Configurator, BasePackage } from 'substance'
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

    this.import(SheetPackage)
  }
}

export default SheetConfigurator
