import { Component } from 'substance'

export default class SheetIssuesCounter extends Component {
  constructor(...args) {
    super(...args)

    this.issueManager = this.context.issueManager
  }

  getInitialState() {
    return {
      showConsole: false
    }
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
    let isActive = this.state.showConsole
    let Button = this.getComponent('button')

    let btn = $$(Button, {
      active: isActive,
      disabled: !hasIssues,
      theme: 'light'
    }).addClass('se-toggle-issues-list')

    let stats = this.issueManager.getStats()

    let errorBtn = $$('span').addClass('se-icon se-errors-counter').append(
      this.renderIcon($$, 'toggle-errors'),
      $$('span').addClass('se-label').append(stats.errors)
    )
    if(stats.errors > 0) errorBtn.addClass('sm-highlighted')

    let warningBtn = $$('span').addClass('se-icon se-errors-counter').append(
      this.renderIcon($$, 'toggle-warnings'),
      $$('span').addClass('se-label').append(stats.warnings)
    )
    if(stats.warnings > 0) warningBtn.addClass('sm-highlighted')

    btn.append(
      errorBtn,
      warningBtn,
      $$('span').addClass('se-icon se-info-counter').append(
        this.renderIcon($$, 'toggle-info'),
        $$('span').addClass('se-label').append(stats.info)
      )
    ).on('click', this.onClick)

    el.append(btn)

    return el
  }

  renderIcon($$, icon) {
    let iconEl = this.context.iconProvider.renderIcon($$, icon)
    return iconEl
  }

  onClick() {
    let sheetEditor = this.context.app.getSheetEditor()
    if (sheetEditor) {
      sheetEditor.toggleConsole('sheet-issues')
      let showConsole = this.state.showConsole
      this.extendState({showConsole: !showConsole})
    }
  }
}
