import { NodeComponent } from 'substance'
import { getLabel } from 'substance-texture'

export default class ReproFigComponent extends NodeComponent {

  render($$) {
    const node = this.props.node

    let el = $$('div')
      .addClass('sc-'+node.type)
      .attr('data-id', node.id)

    let label = getLabel(node)
    let labelEl = $$('div').addClass('se-label').text(label)
    el.append(labelEl)

    const figType = this._getContentType()
    const content = node.findChild(figType)
    let contentEl
    if (content) {
      contentEl = $$(this.getComponent(figType), {
        node: content,
        disabled: true // HACK: in reader we always want to disable
      })
      el.append(contentEl.ref('content'))
    }

    const title = node.findChild('title')
    let titleEl = $$(this.getComponent('text-property-editor'), {
      path: title.getPath(),
      disabled: true, // HACK: in reader we always want to disable
      placeholder: 'Enter Title'
    }).addClass('se-title').ref('title')
    el.append(titleEl)

    const caption = node.findChild('caption')
    let captionEl
    if (caption) {
      captionEl = $$(this.getComponent('caption'), {
        node: caption,
        disabled: true // HACK: in reader we always want to disable
      })
    }
    el.append(captionEl.ref('caption'))
    return el
  }

  _getContentType() {
    return 'cell'
  }

  _onLabelsChanged(refType) {
    if (refType === this.props.node.type) {
      this.rerender()
    }
  }

}
