import { Configurator, BasePackage, FindAndReplacePackage } from 'substance'

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
import BlockquotePackage from './nodes/blockquote/BlockquotePackage'
import InputSettingsBarPackage from './input-settings-bar/InputSettingsBarPackage'
import DefaultPackage from './nodes/default/DefaultPackage'
import MathPackage from './nodes/math/MathPackage'
import SelectPackage from './nodes/select/SelectPackage'
import RangeInputPackage from './nodes/range-input/RangeInputPackage'
import VegaLitePackage from './vega-lite/VegaLitePackage'
import DocumentModel from './DocumentModel'

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
      DocumentClass: DocumentModel,
      name: 'stencila-document',
      defaultTextType: 'paragraph'
    })

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
    this.import(MathPackage)
    this.import(SelectPackage)
    this.import(RangeInputPackage)
    this.import(FindAndReplacePackage)

    // this.import(ImagePackage)
    this.import(BlockquotePackage)
    // this.import(CodeblockPackage)
    this.import(CellPackage)
    this.import(InlineCellPackage)
    this.import(DefaultPackage)
    this.import(InputSettingsBarPackage)
    this.import(VegaLitePackage)

    this.addIcon('settings', { 'fontawesome': 'fa-cog' })
    this.addLabel('settings', 'Settings')

    // Overlay configuration
    this.addToolPanel('main-overlay', [
      // Displays prompts such as EditLinkTool, which are exclusive
      // so that's why we put them first
      {
        name: 'prompt',
        type: 'tool-prompt',
        commandGroups: ['prompt']
      },
      /*{
        // used to resolve icons and labels
        name: 'text-types',
        type: 'tool-dropdown',
        showDisabled: false,
        contextual: true,
        style: 'minimal',
        commandGroups: ['text-types']
      },*/
      {
        name: 'annotations',
        type: 'tool-group',
        contextual: true,
        showDisabled: false,
        style: 'minimal',
        commandGroups: ['annotations']
      },/*
      {
        name: 'insert',
        type: 'tool-dropdown',
        contextual: true,
        showDisabled: false,
        style: 'minimal',
        commandGroups: ['insert']
      }*/
    ])

    this.addToolPanel('toolbar', [
      {
        name: 'text-types',
        type: 'tool-dropdown',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['text-types']
      },
      /*{
        name: 'annotations',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['annotations']
      },*/
      {
        name: 'insert',
        type: 'tool-dropdown',
        showDisabled: true,
        style: 'descriptive',
        commandGroups: ['insert']
      }
    ])

    this.addToolPanel('workflow', [
      {
        name: 'workflow',
        type: 'tool-group',
        commandGroups: ['workflows']
      }
    ])
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
