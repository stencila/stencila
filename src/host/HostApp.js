import Component from 'substance/ui/Component'

import HostClient from './HostClient'

const packageNames = {
  'node': 'Node',
  'py': 'Python',
  'r': 'R'
}

class HostApp extends Component {

  getInitialState () {
    let data = this.props.data
    let host = new HostClient(data ? data.url : this.props.url)
    return {
      host: host,
      data: data
    }
  }

  render ($$) {
    let host = this.state.host
    let data = this.state.data

    let el = $$('div').addClass('sc-host ui container')

    el.append(
      $$('div').text((packageNames[data.package] || '') + ' Host '),
      $$('div').text(data.id)
    )

    el.append(
      $$('form').addClass('se-form ui form').append(
        $$('div').addClass('ui field').append(
          $$('input').attr({
            type: 'text',
            name: 'address',
            placeholder: 'Enter a component address e.g. +document'
          })
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
    )

    if (data) {
      let components = $$('div').addClass('components ui list')
      if (data.components) {
        for (let component of data.components) {
          components.append(
            $$('div').addClass('ui item').append(
              $$('i').addClass('ui tag icon'),
              $$('div').addClass('ui content').append(
                $$('div').addClass('ui header').append(
                  $$('a').addClass('ui link').attr({
                    href: component.url,
                    target: '_blank'
                  }).text(
                    component.title || component.short || component.address || 'Remote'
                  )
                ),
                $$('span').text(component.type || '')
              )
            )
          )
        }
      }
      el.append(components)

      let schemes = $$('div').addClass('schemes ui horizontal list')
      if (data.schemes) {
        for (let scheme in data.schemes) {
          let details = data.schemes[scheme]
          schemes.append(
            $$('div').addClass('ui item').append(
              $$('div').addClass('ui content').append(
                $$('span').text(scheme),
                $$('i').addClass('ui icon ' + (details.enabled ? 'green check circle' : 'red minus circle'))
              )
            )
          )
        }
      }
      el.append(
        $$('div').addClass('ui segment')
          .append(
            $$('h4').text('Schemes'),
            schemes
          )
      )

      let types = $$('div').addClass('ui horizontal list')
      if (data.types) {
        for (let type in data.types) {
          let details = data.types[type]
          types.append(
            $$('div').addClass('ui item').append(
              $$('div').addClass('ui content').append(
                $$('span').text(type)
              )
            )
          )
        }
      }
      el.append(
        $$('div').addClass('types ui segment')
          .append(
            $$('h4').text('Types'),
            types
          )
      )

      let peers = $$('div').addClass('peers ui list')
      if (data.peers) {
        for (let peer of data.peers) {
          peers.append(
            $$('div').addClass('ui item').append(
              $$('i').addClass('ui tag icon'),
              $$('div').addClass('ui content').append(
                $$('div').addClass('ui header').append(
                  $$('a').addClass('ui link').attr({
                    href: peer.url,
                    target: '_blank'
                  }).text(
                    packageNames[peer.package] || peer.package
                  )
                ),
                $$('span').text(peer.version || '')
              )
            )
          )
        }
      }
      el.append(
        $$('div').addClass('ui segment')
          .append(
            $$('h4').text('Peers'),
            peers
          )
      )
    }

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

}

export default HostApp
