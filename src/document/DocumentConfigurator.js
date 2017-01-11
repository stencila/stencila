import Configurator from 'substance/util/Configurator'

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
import MathPackage from './nodes/math/MathPackage'
import EmojiPackage from './nodes/emoji/EmojiPackage'

import ImagePackage from './nodes/image/ImagePackage'
import BlockquotePackage from './nodes/blockquote/BlockquotePackage'
import CodeblockPackage from './nodes/codeblock/CodeblockPackage'

import InputPackage from './nodes/input/InputPackage'
import SelectPackage from './nodes/select/SelectPackage'
import ExecutePackage from './nodes/execute/ExecutePackage'
import PrintPackage from './nodes/print/PrintPackage'

import MarkPackage from './nodes/mark/MarkPackage'
import DiscussionPackage from './nodes/discussion/DiscussionPackage'
import CommentPackage from './nodes/comment/CommentPackage'

import DefaultPackage from './nodes/default/DefaultPackage'

import SessionPackage from '../shared/nodes/session/SessionPackage'

import ButtonPackage from 'substance/packages/button/ButtonPackage'
import ToolsPackage from './tools/ToolsPackage'
import VisualEditorPackage from './editors/visual/VisualEditorPackage'

/**
 * A "configurator" for a document.
 *
 * Uses the Substance package mechanism to reduce repetition.
 * See `substance/util/AbstractConfigurator` for inherited methods
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
    this.import(MathPackage)
    this.import(EmojiPackage)

    this.import(ImagePackage)
    this.import(BlockquotePackage)
    this.import(CodeblockPackage)

    this.import(InputPackage)
    this.import(SelectPackage)
    this.import(ExecutePackage)
    this.import(PrintPackage)

    this.import(MarkPackage)
    this.import(DiscussionPackage)
    this.import(CommentPackage)

    this.import(SessionPackage)

    this.import(DefaultPackage)

    // Import UI packages
    this.import(ButtonPackage)
    this.import(ToolsPackage)
    this.import(VisualEditorPackage)

    // Icons, not defined in above packages but used in toolsets
    this.addIcon('heading', { 'fontawesome': 'fa-header' })
    this.addIcon('paragraph', { 'fontawesome': 'fa-paragraph' })
    this.addIcon('list', { 'fontawesome': 'fa-list' })
    this.addIcon('table', { 'fontawesome': 'fa-table' })
    this.addIcon('blockquote', { 'fontawesome': 'fa-quote-right' })
    this.addIcon('codeblock', { 'fontawesome': 'fa-code' })
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
