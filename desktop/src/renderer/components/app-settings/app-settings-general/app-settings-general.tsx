import { Component, h, State } from '@stencil/core'
import { i18n } from '../../../../i18n'
import { CHANNEL } from '../../../../preload'

@Component({
  tag: 'app-settings-general',
  styleUrl: 'app-settings-general.css',
  scoped: true,
})
export class AppSettingsGeneral {
  @State() config: Record<string, unknown>

  private updateSetting = (key: string) => (e: Event) => {
    e.preventDefault()
    const target = e.target as HTMLInputElement
    const value = target.checked ?? target.value

    window.api.invoke(CHANNEL.SET_APP_CONFIG, { key, value })
  }

  async componentWillLoad() {
    // TODO: Subscribe to config change events
    const config = await window.api.invoke(CHANNEL.READ_APP_CONFIG)
    this.config = config as Record<string, unknown>
  }

  render() {
    return (
      <div class="appSettingsGeneral">
        <div class="title">
          <h1>{i18n.t('settings.general.title')}</h1>
        </div>

        <form>
          <fieldset>
            <input
              id="errorReporting"
              type="checkbox"
              defaultChecked={(this.config.REPORT_ERRORS as boolean) ?? false}
              onChange={this.updateSetting('REPORT_ERRORS')}
            />

            <label htmlFor="errorReporting">
              {i18n.t('settings.general.crashReports.label')}
            </label>

            <p class="helpText">
              {i18n.t('settings.general.crashReports.help')}
            </p>
          </fieldset>
        </form>
      </div>
    )
  }
}
