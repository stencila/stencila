import { AbstractEditor, ContainerEditor, WorkflowPane, SplitPane } from 'substance'
import CellEngine from './CellEngine'

/**
  The Stencila Document Editor
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

    let ScrollPane = this.componentRegistry.get('scroll-pane')
    let Overlay = this.componentRegistry.get('overlay')
    let Layout = this.componentRegistry.get('layout')
    let Dropzones = this.componentRegistry.get('dropzones')

    // Toggle classes to match properties
    // ['naked', 'reveal', 'edit'].forEach(item => {
    //   if (this.props[item]) el.addClass('sm-' + item)
    // })

    el.append(

      $$(SplitPane, {splitType: 'horizontal', sizeB: 'inherit'}).append(
        $$(ScrollPane, {
          scrollbarType: 'substance',
          scrollbarPosition: 'right'
        })
          .ref('scrollPane')
          .append(
            $$(Overlay, {
              toolPanel: configurator.getToolPanel('main-overlay'),
              theme: 'dark'
            }).ref('overlay'),
            $$(Dropzones),
            $$(Layout, {
              width: 'large'
            }).append(
              // A  ContainerEditor for the content of the document
              $$(ContainerEditor, {
                containerId: 'content',
                disabled: !this.props.edit
              }).ref('containerEditor')
            )
          ),
        $$(WorkflowPane, {
          toolPanel: configurator.getToolPanel('workflow')
        })
      )
      // A `ScrollPane` to manage overlays and other positioning
    )
    return el
  }

}
