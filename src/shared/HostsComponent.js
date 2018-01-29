import { Component } from 'substance'

export default class HostsComponent extends Component {

  constructor(...args) {
    super(...args)

    let host = this.context.host
    host.on('peer:registered', () => this.rerender())
  }

  getInitialState() {
    let host = this.context.host
    return {
      discover: host.options.discover >= 0
    }
  }

  render($$) {
    let host = this.context.host
    let peers = host.peers

    let el = $$('div').addClass('sc-hosts')

    Object.keys(peers).forEach(peerKey => {
      el.append(
        this.renderPeer($$, peers[peerKey], peerKey)
      )
    })

    el.append(
      $$('div').addClass('se-options').append(
        $$('div').append(
          $$('span').addClass('se-label').append('Add a host'),
          $$('input').addClass('se-input').ref('urlInput')
            .on('keyup', this._onHostAdd)
        ),
        $$('div').append(
          $$('span').addClass('se-label').append('Auto-discover hosts'),
          $$('input').attr({type: 'checkbox'}).addClass('se-checkbox')
            .attr(this.state.discover ? 'checked' : 'unchecked', true)
            .on('change', this._onDiscoverToggle)
        )
      )
    )

    return el
  }

  renderPeer($$, peer, name) {
    const contexts = this._getPeerContexts(peer)

    let el = $$('div').addClass('se-host-item').append(
      $$('div').addClass('se-name').append(name)
    )

    contexts.forEach(context => {
      el.append(
        $$('div').addClass('se-context').append(context)
      )
    })

    return el
  }

  _getPeerContexts(peer) {
    const types = peer.types
    let contexts = []
    Object.keys(types).forEach(type => {
      if(types[type].base !== 'Storer') contexts.push(types[type].name)
    })
    return contexts
  }

  _onHostAdd(e) {
    if (e.keyCode === 13) {
      const urlInput = this.refs.urlInput
      const url = urlInput.val()
      let host = this.context.host
      host.pokePeer(url)
    }
  }

  _onDiscoverToggle() {
    let host = this.context.host
    if (this.state.discover) {
      host.discoverPeers(-1)
      this.setState({discover: false})
    } else {
      host.discoverPeers(10)
      this.setState({discover: true})
    }
  }

}
