import { Configurator, BasePackage } from 'substance'

import TitlePackage from './nodes/title/TitlePackage'
import SummaryPackage from './nodes/summary/SummaryPackage'
import HeadingPackage from './nodes/heading/HeadingPackage'
import ParagraphPackage from './nodes/paragraph/ParagraphPackage'
import EmphasisPackage from './nodes/emphasis/EmphasisPackage'
import StrongPackage from './nodes/strong/StrongPackage'
import SubscriptPackage from './nodes/subscript/SubscriptPackage'
import SuperscriptPackage from './nodes/superscript/SuperscriptPackage'
import CodePackage from './nodes/code/CodePackage'
import LinkPackage from './nodes/link/LinkPackage'
import ListPackage from './nodes/list/ListPackage'
import CellPackage from './nodes/cell/CellPackage'
import InlineCellPackage from './nodes/inline-cell/InlineCellPackage'
import TablePackage from './nodes/table/TablePackage'
import BlockquotePackage from './nodes/blockquote/BlockquotePackage'
import CodeblockPackage from './nodes/codeblock/CodeblockPackage'
import MinimalSwitchTextTypePackage from './minimal-switch-text-type/MinimalSwitchTextTypePackage'
import InputSettingsBarPackage from './input-settings-bar/InputSettingsBarPackage'
import DefaultPackage from './nodes/default/DefaultPackage'
import ToggleInsertPackage from './toggle-insert/ToggleInsertPackage'
import MathPackage from './nodes/math/MathPackage'
// import ImagePackage from './nodes/image/ImagePackage'
import SelectPackage from './nodes/select/SelectPackage'
import RangeInputPackage from './nodes/range-input/RangeInputPackage'
import VegaLitePackage from './vega-lite/VegaLitePackage'

/**
 * A "configurator" for a document.
 *
 * Uses the Substance package mechanism to reduce repetition.
 * See `substance.Configurator` for inherited methods
 * used by `DocumentHTMLImporter`, `DocumentEditor` etc
 *
 * @class      DocumentConfigurator (name)
 */
class DocumentConfigurator extends Configurator {

  constructor () {
    super()

    // Define the schema (used by `getSchema()` to generate a `DocumentSchema` based on this
    // and the nodes added below by imports)
    this.defineSchema({
      name: 'stencila-document',
      defaultTextType: 'paragraph'
    })

    // At present, need at least the 'default' tool group before adding tools via imports below
    this.addToolGroup('default')

    this.import(BasePackage)
    // Import node packages, in "order of appearance"
    this.import(TitlePackage)
    this.import(SummaryPackage)
    this.import(HeadingPackage)
    this.import(ParagraphPackage)
    this.import(EmphasisPackage)
    this.import(StrongPackage)
    this.import(SubscriptPackage)
    this.import(SuperscriptPackage)
    this.import(CodePackage)
    this.import(LinkPackage)
    this.import(ListPackage)
    this.import(TablePackage)
    this.import(MathPackage)
    this.import(SelectPackage)
    this.import(RangeInputPackage)

    // this.import(ImagePackage)
    this.import(BlockquotePackage)
    this.import(CodeblockPackage)
    this.import(CellPackage)
    this.import(InlineCellPackage)
    this.import(DefaultPackage)
    this.import(MinimalSwitchTextTypePackage)
    this.import(ToggleInsertPackage)
    this.import(InputSettingsBarPackage)
    this.import(VegaLitePackage)

    this.addIcon('settings', { 'fontawesome': 'fa-cog' })
    this.addLabel('settings', 'Settings')

    // this.addKeyboardShortcut('alt+ENTER', { command: 'insert-cell' })
  }

  /**
   * Gets the file client
   *
   * Method required by `AbstractEditor._initialize`
   *
   * @return     {<type>}  The file client.
   */
  getFileClient () {
    return null
  }

  /**
   * Gets the save handler.
   *
   * Method required by `AbstractEditor._initialize`
   *
   * @return     {<type>}  The save handler.
   */
  getSaveHandler () {
    return null
  }
}

export default DocumentConfigurator
