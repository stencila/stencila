import Component from 'substance/ui/Component'

import ComponentDelegate from '../ComponentDelegate'

class FolderApp extends Component {

  getInitialState () {
    let data = this.props.data
    let delegate = new ComponentDelegate(data ? data.url : this.props.url)
    return {
      delegate: delegate,
      data: data,
      list: null
    }
  }

  render ($$) {
    let data = this.state.data
    let url = data.url
    let list = this.state.list

    let el = $$('div').addClass('ui container')

    el.append(
      $$('h1').append(
        $$('i').addClass('ui icon folder'),
        $$('span').text(data.short || data.address)
      )
    )

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
    this.state.delegate.show().then(data => {
      this.state.delegate.call('list').then(list => {
        this.extendState({
          data: data,
          list: list
        })
      })
    })
  }

}

export default FolderApp
