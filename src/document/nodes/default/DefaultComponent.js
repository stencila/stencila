import Component from 'substance/ui/Component'

import CodeEditorComponent from '../../ui/CodeEditorComponent'
import sanitize from './sanitize'

class DefaultComponent extends Component {

  getInitialState () {
    return {
      edit: false
    }
  }

  render ($$) {
    var node = this.props.node
    var el = super.render.call(this, $$)
      .addClass('sc-default')
    if (this.state.edit) {
      var code = $$(CodeEditorComponent, {
        node: node,
        codeProperty: 'html',
        languageProperty: null,
        language: 'html'
      }).ref('code')
      el.append(code)
    }
    return el.append(
      $$('div')
        .ref('display')
        .addClass('se-display')
        .attr('contenteditable', false)
        .html(sanitize(node.html))
    )
  }

  didMount () {
    this.props.node.on('html:changed', this.rerender, this)
    this.props.node.on('edit:toggle', this._onEditToggle, this)
  }

  _onEditToggle () {
    this.extendState({
      edit: !this.state.edit
    })
  }

}

DefaultComponent.fullWidth = true

export default DefaultComponent
