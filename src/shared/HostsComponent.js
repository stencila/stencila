import { Component } from 'substance'

export default class HostsComponent extends Component {

  constructor(...args) {
    super(...args)

    let host = this.context.host
    host.on('peer:registered', () => this.rerender())
    host.on('instance:created', () => this.rerender())
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

    el.append(
      this.renderHost($$, host, host, 'internal')
    )
    Object.keys(peers).forEach(url => {
      el.append(
        this.renderHost($$, host, peers[url], url)
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

  renderHost($$, internalHost, host, url) {
    let el = $$('div').addClass('se-host-item').append(
      $$('div').addClass('se-name').append(url)
    )

    let types = host.types || {}
    if (url !== 'internal') {
      let peers = host.peers || {}
      for (let key of Object.keys(peers)) {
        types = Object.assign(types, peers[key].types || {})
      }
    }
    const instances = internalHost.instances
    for (let type of Object.keys(types)) {
      if(types[type].base === 'Storer') continue
      let instantiated = false
      for (let key of Object.keys(instances)) {
        let instance = instances[key]
        if (instance.type === type && instance.host === url) {
          instantiated = true
          break
        }
      }
      el.append(
        $$('div').addClass('se-type').addClass(instantiated ? 'sm-instantiated': '')
          .text(type)
      )
    }

    return el
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
