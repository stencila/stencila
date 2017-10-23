import { NodeComponent } from 'substance'
import { REF_TYPES } from 'substance-texture'

export default class ReproFigComponent extends NodeComponent {

  didMount() {
    super.didMount()
    this.context.labelGenerator.on('labels:generated', this._onLabelsChanged, this)
  }

  dispose() {
    super.dispose()
    this.context.labelGenerator.off(this)
  }

  render($$) {
    const node = this.props.node
    const labelGenerator = this.context.labelGenerator

    let el = $$('div')
      .addClass('sc-'+node.type)
      .attr('data-id', node.id)

    let label = labelGenerator.getLabel(REF_TYPES[node.type], [node.id])
    let labelEl = $$('div').addClass('se-label').text(label)
    el.append(labelEl)

    const figType = this._getContentType()
    const content = node.findChild(figType)
    let contentEl
    if (content) {
      contentEl = $$(this.getComponent(figType), {
        node: content,
        disabled: this.props.disabled
      })
      el.append(contentEl.ref('content'))
    }

    const title = node.findChild('title')
    let titleEl = $$(this.getComponent('text-property-editor'), {
      path: title.getPath(),
      disabled: this.props.disabled
    }).addClass('se-title').ref('title')
    el.append(titleEl)

    const caption = node.findChild('caption')
    let captionEl
    if (caption) {
      captionEl = $$(this.getComponent('caption'), {
        node: caption,
        disabled: this.props.disabled
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
