import Component from 'substance/ui/Component'

import HostClient from './HostClient'

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
      let components = $$('div').addClass('ui list')
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
                  component.title || component.address
                )
              ),
              $$('span').text(component.type || '')
            )
          )
        )
      }
      el.append(components)

      let schemes = $$('div').addClass('ui horizontal list')
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
      el.append(
        $$('div').addClass('ui segment')
          .append(schemes)
      )

      let formats = $$('div').addClass('ui horizontal list')
      for (let format of data.formats) {
        formats.append(
          $$('div').addClass('ui item').append(
            $$('div').addClass('ui content').append(
              $$('span').text(format)
            )
          )
        )
      }
      el.append(formats)

      let peers = $$('div').addClass('ui horizontal list')
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
                  {
                    'py': 'Python',
                    'r': 'R'
                  }[peer.package] || peer.package
                )
              ),
              $$('span').text(peer.version || '')
            )
          )
        )
      }
      el.append(peers)
      el.append(
        $$('button').addClass('ui button')
          .text('Discover')
          .on('click', () => {
            host.discover().then(() => {
              this.refresh()
            })
          })
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
