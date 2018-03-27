import { Component, FontAwesomeIcon } from 'substance'

export default class HostsComponent extends Component {

  constructor(...args) {
    super(...args)

    let host = this.context.host
    host.on('environs:registered', () => this.rerender())
    host.on('environ:selected', () => this.rerender())
    host.on('environ:authenticate', (environ) => this._onEnvironAuthenticate(environ))
    host.on('instance:created', () => this.rerender())
  }

  getInitialState() {
    let host = this.context.host
    return {
      environ: null,
      hostAddShow: false,
      discover: host.options.discover >= 0
    }
  }

  render($$) {
    let host = this.context.host
    let environs = host.mapEnvirons()
    let selectedEnviron = this.state.environ || Object.keys(environs)[0]

    let el = $$('div').addClass('sc-hosts')

    let environEl = $$('div').addClass('se-environ').append(
      $$('div').addClass('se-label').append('Select an execution environment:')
    )
    if (Object.keys(environs).length) {
      let environSelect = $$('select').addClass('se-environ-select')
        .ref('environSelect')
        .on('change', this._onEnvironChange)
      Object.keys(environs).sort().forEach(environ => {
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
    let hostList = $$('div').addClass('se-host-list')
    // List the hosts that provide the selected environment
    let environsHosts = environs[selectedEnviron]
    if (environsHosts) {
      environsHosts.forEach(environ => {
        let name = environ.origin
        let match = environ.origin.match(/^https?:\/\/([^:]+)(:(\d+))?/)
        if (match) {
          let domain = match[1]
          if (domain === '127.0.0.1') domain = 'localhost'
          name = domain
          let port = match[3]
          if(port) name += ':' + port
        }
        if (environ.path.length) name += '/' + environ.path
        let hostItem = $$('div').addClass('se-host-item').append(
          $$('div').addClass('se-name').append(name)
        ).on('click', this._onEnvironSelected.bind(this, environ))
        if (environ.selected) hostItem.addClass('sm-selected')
        hostList.append(hostItem)
      })
    } else {
      hostList.append(
        $$('div').addClass('se-message').text(`No registered hosts provide ${this.state.environ}`)
      )
    }
    hostsEl.append(hostList)
    if (true) {//(this.state.hostAddShow) {
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
    } else {
      hostsEl.append(
        $$('button').addClass('se-button')
          .append($$(FontAwesomeIcon, {icon: 'fa-plus' }))
          .append('Add host')
          .on('click', this._onHostAddToggle)
      )
    }

    el.append(
      environEl,
      hostsEl
    )
    return el
  }

  _onEnvironChange() {
    const environSelect = this.refs.environSelect
    const environ = environSelect.val()
    this.setState({environ: environ})
  }

  _onEnvironSelected(environ) {
    let host = this.context.host
    host.selectEnviron(environ)
  }

  _onEnvironAuthenticate(environ) {
    console.log('TODO: Prompt for key for environ with uuid: ' + environ.uuid)
  }

  _onHostAddToggle() {
    if (this.state.hostAddShow) {
      this.setState({hostAddShow: false})
    } else {
      this.setState({hostAddShow: true})
    } 
  }

  _onHostAdd() {
    const urlInput = this.refs.urlInput
    const url = urlInput.val()
    const keyInput = this.refs.keyInput
    const key = keyInput.val()
    let host = this.context.host
    host.registerPeer(url, key)
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
