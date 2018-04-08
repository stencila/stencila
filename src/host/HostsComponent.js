import { Component } from 'substance'

export default class HostsComponent extends Component {

  constructor(...args) {
    super(...args)

    let host = this.context.host
    host.on('environ:changed', () => this.rerender())
    host.on('hosts:changed', () => this.rerender())
    host.on('peers:changed', () => this.rerender())
  }

  getInitialState() {
    let host = this.context.host
    return {
      hostAddShow: false,
      discover: host.options.discover >= 0
    }
  }

  render($$) {
    let host = this.context.host
    
    // Generate a list of available enviroments from the
    // registered hosts
    let availableEnvirons = {}
    for (let otherHost of host.hosts.values()) {
      for (let environ of otherHost.manifest.environs) {
        availableEnvirons[environ.id] = environ
      }
    }

    let selectedEnviron = host._environ || Object.keys(availableEnvirons)[0]

    let el = $$('div').addClass('sc-hosts')

    let environEl = $$('div').addClass('se-environ').append(
      $$('div').addClass('se-label').append('Select an execution environment:')
    )
    if (Object.keys(availableEnvirons).length) {
      let environSelect = $$('select').addClass('se-environ-select')
        .ref('environSelect')
        .on('change', this._onEnvironChange)
      Object.keys(availableEnvirons).sort().forEach(environ => {
        let option = $$('option').attr('value', environ).text(environ)
        if (selectedEnviron === environ) option.attr('selected', 'true')
        environSelect.append(option)
      })
      environEl.append(environSelect)
    } else {
      environEl.append(
        $$('div').addClass('se-message').text('No execution environments have been registered. Please add an execution host first.')
      )
    }

    let hostsEl = $$('div').addClass('se-hosts').append(
      $$('span').addClass('se-label').append('Select a host for environment:')
    )
    if (host.hosts.size) {
      let hostList = $$('div').addClass('se-host-list')
      for (let [url, otherHost] of host.hosts) {
        let name = url
        let match = url.match(/^https?:\/\/([^:]+)(:(\d+))?/)
        if (match) {
          let domain = match[1]
          if (domain === '127.0.0.1') domain = 'localhost'
          name = domain
          let port = match[3]
          if(port) name += ':' + port
        }
        let nameEl = $$('div').addClass('se-name').append(name)

        let environsEl = $$('div').addClass('se-host-environs')
        for (let environ of otherHost.manifest.environs) {
          environsEl.append($$('span').addClass('se-host-environ').append(environ.id))
        }

        let hostItem = $$('div').addClass('se-host-item').append(
          nameEl, environsEl
        ).on('click', this._onHostClick.bind(this, url, otherHost))
        if (otherHost.selected) hostItem.addClass('sm-selected')
        hostList.append(hostItem)
      }
      hostsEl.append(hostList)
    } else {
      hostsEl.append(
        $$('div').addClass('se-message').text(`No registered hosts provide ${selectedEnviron}`)
      )
    }
    
    /*
    hostsEl.append(
      $$('div').addClass('se-host-add').append(
        $$('div').append(
          $$('span').addClass('se-label').append('Add a host'),
          $$('input').addClass('se-input').attr({'placeholder': 'URL e.g. http://127.0.0.1:2100'})
            .ref('urlInput'),
          $$('input').addClass('se-input').attr({'placeholder': 'Key'})
            .ref('keyInput'),
          $$('button').addClass('se-button')
            .text('Add')
            .on('click', this._onHostAdd)
        ),
        $$('div').append(
          $$('span').addClass('se-label').append('Auto-discover hosts'),
          $$('input').attr({type: 'checkbox'}).addClass('se-checkbox')
            .attr(this.state.discover ? 'checked' : 'unchecked', true)
            .on('change', this._onDiscoverToggle)
        )
      )
    )
    */
   
    let peersEl = $$('div').addClass('se-peers').append(
      $$('span').addClass('se-label').append('Connected execution environments:')
    )
    if (host.peers.size) {
      let peerList = $$('div').addClass('se-peer-list')
      for (let [url, manifest] of host.peers) {
        let name = url
        let match = url.match(/^https?:\/\/([^:]+)(:(\d+))?/)
        if (match) {
          let domain = match[1]
          if (domain === '127.0.0.1') domain = 'localhost'
          name = domain
          let port = match[3]
          if(port) name += ':' + port
        }
        name += '/' + (manifest.environs && manifest.environs[0] && manifest.environs[0].id)
        let nameEl = $$('div').addClass('se-name').append(name)

        let contextsEl = $$('div').addClass('se-peer-contexts')
        for (let name of Object.keys(manifest.types)) {
          contextsEl.append($$('span').addClass('se-peer-context').append(name))
        }

        peerList.append($$('div').addClass('se-peer-item').append(
          nameEl, contextsEl
        ))
      }
      peersEl.append(peerList)
    } else {
      peersEl.append(
        $$('div').addClass('se-message').text('No external execution environments connected')
      )
    }

    el.append(environEl, hostsEl, peersEl)
    return el
  }

  _onEnvironChange() {
    let host = this.context.host
    const environSelect = this.refs.environSelect
    const environ = environSelect.val()
    host.selectEnviron(environ)
  }

  _onHostClick(url, otherHost) {
    let host = this.context.host
    if (!otherHost.selected) host.selectHost(url)
    else host.deselectHost(url)
  }

  _onHostAdd() {
    const urlInput = this.refs.urlInput
    const url = urlInput.val()
    const keyInput = this.refs.keyInput
    const key = keyInput.val()
    let host = this.context.host
    host.registerHost(url, key)
  }

  _onDiscoverToggle() {
    let host = this.context.host
    if (this.state.discover) {
      host.discoverHosts(-1)
      this.setState({discover: false})
    } else {
      host.discoverHosts(10)
      this.setState({discover: true})
    }
  }

}
