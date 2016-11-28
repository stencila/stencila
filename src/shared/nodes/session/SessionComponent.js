import Component from 'substance/ui/Component'

class SessionComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = super.render.call(this, $$)
      .addClass('sc-session')
      .html('<pre>Work in progress\n' + JSON.stringify(this.node) + '</pre>')
    return el
  }

}

export default SessionComponent
