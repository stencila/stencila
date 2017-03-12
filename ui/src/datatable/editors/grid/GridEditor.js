import AbstractEditor from 'substance/ui/AbstractEditor'
import ScrollPane from 'substance/packages/scroll-pane/ScrollPane'
import ContainerEditor from 'substance/ui/ContainerEditor'

class GridEditor extends AbstractEditor {

  render ($$) {
    var configurator = this.props.configurator

    var el = $$('div').addClass('sc-grid-editor')

    el.append(
      $$(ScrollPane, {
        scrollbarType: 'native',
        scrollbarPosition: 'right'
      })
        .ref('scrollPane')
        .append(
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

export default GridEditor
