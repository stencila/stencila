import { AbstractEditor, ScrollPane, ContainerEditor } from 'substance'
import MacroManager from './ui/MacroManager'

/**
 * A editor for a Stencila Document
 *
 * @class      VisualEditor (name)
 */
export default class DocumentEditor extends AbstractEditor {

  constructor (...args) {
    super(...args)
    let configurator = this.getConfigurator()
    // Use custom MacroManager
    this.macroManager.context.editorSession.off(this.macroManager)
    delete this.macroManager
    this.macroManager = new MacroManager(this.props.editorSession._context, configurator.getMacros())
  }

  /**
   * Render this editor
   */
  render ($$) {
    var configurator = this.getConfigurator()
    var el = $$('div').addClass('sc-visual-editor')

    // Toggle classes to match properties
    // ['naked', 'reveal', 'edit'].forEach(item => {
    //   if (this.props[item]) el.addClass('sm-' + item)
    // })

    // Document toolset (becuase of the way in which
    // tools and commands work, this has to go here, under an `AbstractEditor`,
    // instead of under the `DocumentApp`)
    // el.append(
    //   $$(DocumentToolset, {
    //     copy: this.props.copy,
    //     view: this.props.view,
    //     reveal: this.props.reveal,
    //     comment: this.props.comment,
    //     edit: this.props.edit
    //   }).ref('documentToolset')
    // )

    el.append(
      // A `ScrollPane` to manage overlays and other positioning
      $$(ScrollPane, {
        scrollbarType: 'native',
        scrollbarPosition: 'right',
        // overlay: Overlayer
      })
        .ref('scrollPane')
        .append(
          // A  ContainerEditor for the content of the document
          $$(ContainerEditor, {
            containerId: 'content',
            disabled: !this.props.edit,
            commands: configurator.getSurfaceCommandNames(),
            textTypes: configurator.getTextTypes()
          }).ref('containerEditor')
        )
    )
    return el
  }

}
