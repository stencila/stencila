import { AbstractEditor, ContainerEditor } from 'substance'
import CellEngine from './CellEngine'
/**
 * The Stencila Document Editor
 *
 * @class      VisualEditor (name)
 */
export default class DocumentEditor extends AbstractEditor {

  constructor (...args) {
    super(...args)

    this._cellEngine = new CellEngine(this.editorSession)
  }

  dispose() {
    super.dispose()

    this._cellEngine.dispose()
  }

  /**
   * Render this editor
   */
  render ($$) {
    var configurator = this.getConfigurator()
    var el = $$('div').addClass('sc-document-editor')

    let BodyScrollPane = this.componentRegistry.get('body-scroll-pane')
    let Overlay = this.componentRegistry.get('overlay')
    let Dropzones = this.componentRegistry.get('dropzones')

    // Toggle classes to match properties
    // ['naked', 'reveal', 'edit'].forEach(item => {
    //   if (this.props[item]) el.addClass('sm-' + item)
    // })

    el.append(
      // A `ScrollPane` to manage overlays and other positioning
      $$(BodyScrollPane)
        .ref('scrollPane')
        .append(
          $$(Overlay, {
            toolPanel: configurator.getToolPanel('main-overlay'),
            theme: 'dark'
          }).ref('overlay'),
          $$(Dropzones),
          // A  ContainerEditor for the content of the document
          $$(ContainerEditor, {
            containerId: 'content',
            disabled: !this.props.edit
          }).ref('containerEditor')
        )
    )
    return el
  }

}
