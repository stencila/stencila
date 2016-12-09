import Component from 'substance/ui/Component'

import HostClient from './HostClient'

const packageNames = {
  'node': 'Node.js',
  'py': 'Python',
  'r': 'R'
}

const kindIcons = {
  folder: 'folder open outline',
  document: 'file text outline',
  session: 'terminal'
}

class HostApp extends Component {

  getInitialState () {
    let data = this.props.data
    let host = new HostClient(data ? data.url : this.props.url)
    return {
      host: host,
      data: data,
      peers: data.peers
    }
  }

  render ($$) {
    let host = this.state.host
    let data = this.state.data

    let el = $$('div').addClass('ui container').css({
      margin: '1em auto'
    })

    // Header
    el.append(
      $$('h1').addClass('ui').css({
        'margin-bottom': '2em'
      }).append(
        $$('i').addClass('ui icon square'),
        data.short || data.address
      )
    )

    // Components section

    let schemes = $$('select')
      .addClass('ui dropdown')
      .css({
        'min-width': '5em'
      })
      .append(
        $$('option').attr('value', '').text('')
      )
    if (data.schemes) {
      for (let scheme in data.schemes) {
        if (scheme !== 'new') {
          // let details = data.schemes[scheme]
          schemes.append(
            $$('option').attr('value', scheme).text(scheme)
          )
        }
      }
    }

    let components = $$('div').addClass('components ui list')
    if (data.components) {
      // Reverse this list so that newest is first
      for (let component of data.components) {
        components.append(
          $$('div').addClass('ui item').append(
            $$('i').addClass('ui icon ' + (kindIcons[component.kind] || 'circle')),
            $$('div').addClass('ui content').append(
              $$('div').addClass('ui header').append(
                $$('a').addClass('ui link').attr({
                  href: component.url,
                  target: '_blank'
                }).text(
                  component.title || component.short || component.address || 'Remote'
                )
              )
              // $$('span').text(component.type || '')
            )
          )
        )
      }
    }

    let form = $$('form').addClass('se-form ui form').append(
        $$('div').addClass('ui labeled input').append(
          schemes,
          $$('input').attr({
            type: 'text',
            name: 'address',
            placeholder: 'Enter a component address e.g. +document'
          }).ref('address')
        )
      ).on('submit', (event) => {
        let address = event.target.address.value
        host.open(address)
          .then(value => {
            this.refresh()
            //window.location = value.url
          })
          .catch(error => {
            console.error(error)
          })
        event.preventDefault()
      })

    /*if (data.types) {
      el.append(
        $$('span').text('new:')
      )
      for (let type in data.types) {
        let details = data.types[type]
        el.append(
          $$('button')
            .addClass('ui small basic icon button')
            .html('<i class="ui file text outline icon"></i>')
            .on('click', () => {
              this.refs['address'].el.el.value = '+' + type
            })
        )
      }
    }*/

    el.append(
      $$('div')
        .addClass('ui top attached message clearfix')
        .append(
          $$('h3')
            .addClass('ui header')
            .css({
              float: 'left'
            })
            .text('Components'),
          $$('button')
            .addClass('ui labeled icon button')
            .css({
              float: 'right'
            })
            .append(
              $$('i').addClass('ui refresh icon'),
              'Refresh'
            )
        ),
      $$('div')
        .addClass('ui bottom attached simple segment')
        .append(form, components)
    )

    // Peers list
    let peers = $$('div').addClass('ui horizontal list')
    if (this.state.peers) {
      for (let peer of this.state.peers) {
        peers.append(
          $$('div').addClass('ui item').append(
            $$('i').addClass('ui square icon'),
            $$('div').addClass('ui content').append(
              $$('div').addClass('ui header').append(
                $$('a').addClass('ui link').attr({
                  href: peer.url || '?',
                  target: '_blank'
                }).text(
                  packageNames[peer.package] || peer.package || '?'
                )
              ),
              $$('span').text(peer.version || '')
            )
          )
        )
      }
    }
    el.append(
      $$('div')
        .addClass('ui top attached message clearfix')
        .css({
          'margin-top': '3em'
        })
        .append(
          $$('h3')
            .addClass('ui header')
            .css({
              float: 'left'
            })
            .text('Peers'),
          $$('button')
            .addClass('ui labeled icon button')
            .css({
              float: 'right'
            })
            .append(
              $$('i').addClass('ui find icon'),
              'Discover'
            )
            .on('click', () => this.discover())
        ),
      $$('div')
        .addClass('ui bottom attached simple segment')
        .append(peers)
    )

    return el
  }

  didMount () {
    if (!this.state.data) this.refresh()
  }

  refresh () {
    this.state.host.show().then(data => {
      this.extendState({
        data: data
      })
    })
  }

  discover () {
    this.state.host.discover().then(peers => {
      this.extendState({
        peers: peers
      })
    })
  }

}

export default HostApp
