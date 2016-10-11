import Component from 'substance/ui/Component'

import RemoteHost from 'stencila/src/host/RemoteHost'

class HostApp extends Component {

  getInitialState () {
    let host = this.props.data
    return {
      host: new RemoteHost(host.url)
    }
  }

  render ($$) {
    let host = this.props.data

    let el = $$('div').addClass('ui container').css({
      'margin-top': '10%'
    })

    el.append(
      $$('form').addClass('ui form').append(
        $$('div').addClass('ui field').append(
          $$('input').attr({
            type: 'text',
            name: 'address',
            placeholder: 'Enter a component address e.g. +document'
          })
        )
      ).on('submit', (event) => {
        let address = event.target.address.value
        this.state.host.open(address)
          .then(function (value) {
            console.log(value)
          })
          .catch(function (error) {
            console.error(error)
          })
        event.preventDefault()
      })
    )

    let components = $$('div').addClass('ui horizontal list')
    for (let component of host.components) {
      components.append(
        $$('div').addClass('ui item').append(
          $$('i').addClass('ui tag icon'),
          $$('div').addClass('ui content').append(
            $$('div').addClass('ui header').append(
              $$('a').addClass('ui link').attr({
                href: component.url,
                target: '_blank'
              }).text(
                component.title || 'Untitled'
              )
            ),
            $$('span').text(component.type || '')
          )
        )
      )
    }
    el.append(components)

    let peers = $$('div').addClass('ui horizontal list')
    for (let peer of host.peers) {
      peers.append(
        $$('div').addClass('ui item').append(
          $$('i').addClass('ui tag icon'),
          $$('div').addClass('ui content').append(
            $$('div').addClass('ui header').append(
              $$('a').addClass('ui link').attr({
                href: peer.url,
                target: '_blank'
              }).text(
                packageNames[peer.package]
              )
            ),
            $$('span').text(peer.version || '')
          )
        )
      )
      console.log(peer)
    }
    el.append(peers)

    return el
  }

}

const packageNames = {
  'py': 'Python'
}

export default HostApp
