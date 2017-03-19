import {Component, Button} from 'substance'

class Dropdown extends Component {

  render($$) {
    let el = $$('div').addClass('sc-dropdown')
    el.append(
      this.renderButton($$)
    )
    if (this.state.open) {
      el.append(
        $$('div').addClass('se-options').append(
          $$('div').addClass('se-arrow'),
          $$('div').addClass('se-content').append(
            this.renderItems($$)
          )
        )
      )
    }
    return el
  }

  renderButton($$) {
    let btn = $$(Button, {
      label: this.props.name,
      icon: this.props.icon,
      active: this.state.open,
      disabled: this.props.disabled,
      style: 'outline'
    }).on('click', this.onClickButton)
    return btn
  }

  renderItems($$) {
    const items = this.props.items
    let els = []
    items.forEach((item) => {
      let itemEl
      switch(item.type) {
        case 'button': {
          itemEl = this.renderButtonItem($$, item)
          break
        }
        case 'choice': {
          itemEl = this.renderChoiceItem($$, item)
          break
        }
        default: {
          itemEl = this.renderButtonItem($$, item)
        }
      }
      els.push(itemEl)
    })
    return els
  }

  renderButtonItem($$, item) {
    return $$(Button, {
      label: item.label,
      icon: item.icon,
      active: item.active !== false,
      disabled: item.disabled === true,
      style: this.props.style
    }).addClass('se-button')
  }

  renderChoiceItem($$, item) {
    const choices = item.choices || []
    let el = $$('div').addClass('se-choice')
    el.append(
      $$('div').addClass('se-label').text(item.label)
    )
    let group = $$('form')
    choices.forEach((choice) => {
      const value = choice.value || choice.label
      let input = $$('input').attr({
        type: 'radio',
        name: item.name,
        value: value,
      })
      if (value === item.value) {
        input.attr('checked', true)
      }
      input.on('change', this.onChangeChoice)
      let choiceEl = $$('div').append(
        input,
        choice.label
      )
      group.append(choiceEl)
    })
    el.append(group)
    return el
  }

  onClickButton() {
    let open = !this.state.open
    this.setState({
      open: open
    })
  }

  onChangeChoice(event) {
    this.el.emit('select', {
      name: event.target.name,
      value: event.target.value
    })
  }
}

export default Dropdown
