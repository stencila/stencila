import Component from 'substance/ui/Component'

import FolderClient from './FolderClient'

class FolderApp extends Component {

  getInitialState () {
    let data = this.props.data
    let client = new FolderClient(data ? data.url : this.props.url)
    return {
      client: client,
      data: data,
      list: null
    }
  }

  render ($$) {
    let data = this.state.data
    let url = data.url
    let list = this.state.list

    let el = $$('div').addClass('ui container').css({
      'margin-top': '10%'
    })

    if (list) {
      let items = $$('div').addClass('ui list')
      for (let item of list) {
        items.append(
          $$('div').addClass('ui item').append(
            $$('div').addClass('ui content').append(
              $$('a').addClass('ui link').attr({
                href: url + '/' + item
              }).text(item)
            )
          )
        )
      }
      el.append(items)
    }

    el.append(
      $$('button').addClass('ui button')
        .text('Refresh')
        .on('click', () => this.refresh())
    )

    return el
  }

  didMount () {
    if (!this.state.data || !this.state.list) this.refresh()
  }

  refresh () {
    this.state.client.show().then(data => {
      this.state.client.call('list').then(list => {
        this.extendState({
          data: data,
          list: list
        })
      })
    })
  }

}

export default FolderApp
