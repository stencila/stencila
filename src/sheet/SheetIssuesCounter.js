import { Component, Tooltip } from 'substance'
const ISSUE_TYPES = ['errors', 'warnings', 'info', 'failed', 'passed']

export default class SheetIssuesCounter extends Component {
  constructor(...args) {
    super(...args)

    this.issueManager = this.context.issueManager
  }

  didMount() {
    this.issueManager.on('issues:changed', this.rerender, this)
  }

  dispose() {
    this.issueManager.off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-issues-counter')
    let hasIssues = this.issueManager.hasAnyIssues()
    let Button = this.getComponent('button')

    let btn = $$(Button, {
      active: false,
      disabled: !hasIssues,
      theme: 'light'
    }).addClass('se-toggle-issues-list')
      .on('click', this.onClick)

    ISSUE_TYPES.forEach(type => {
      btn.append(this.renderItem($$, type))
    })

    el.append(btn)

    return el
  }

  renderIcon($$, icon) {
    let iconEl = this.context.iconProvider.renderIcon($$, icon)
    return iconEl
  }

  renderItem($$, type) {
    const stats = this.issueManager.getStats()
    let itemEl = $$('span').addClass('se-icon se-' + type + '-counter').append(
      this.renderIcon($$, 'toggle-' + type),
      $$('span').addClass('se-label').append(stats[type]),
      $$(Tooltip, {
        text: this.getLabel('toggle-' + type)
      })
    )
    if(stats[type] > 0) itemEl.addClass('sm-highlighted')
    return itemEl
  }

  onClick() {
    let sheetEditor = this.context.app.getSheetEditor()
    if (sheetEditor) {
      sheetEditor.toggleContext('sheet-issues')
    }
  }
}
