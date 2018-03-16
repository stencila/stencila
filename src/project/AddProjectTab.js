import { Component, FontAwesomeIcon } from 'substance'

export default class AddProjectTab extends Component {

  getInitialState() {
    return {
      menu: false
    }
  }

  render($$) {
    // NOTE: We use sc-project-tab here to inherit its styles
    let el = $$('div').addClass('sc-add-project-tab sc-project-tab')

    el.append(
      $$(FontAwesomeIcon, {icon: 'fa-plus-circle'})
    )
    .on('click', this._toggleMenu)

    if (this.state.menu) {
      el.append(
        $$('ul').addClass('se-menu').append(
          $$('li').append('Sheet').on('click', this._addSheet),
          $$('li').append('Article').on('click', this._addArticle)
        )
      )
    }
    return el
  }

  _toggleMenu() {
    this.extendState({ menu: !this.state.menu })
  }

  _addSheet() {
    this.send('addDocument', 'sheet')
  }

  _addArticle() {
    this.send('addDocument', 'article')
  }

}
