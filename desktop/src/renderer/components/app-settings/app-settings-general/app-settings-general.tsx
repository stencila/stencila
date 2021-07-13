import { Component, h, State } from '@stencil/core'
import { i18n } from '../../../../i18n'
import { UnprotectedStoreKeys } from '../../../../preload/stores'
import { client } from '../../../client'

@Component({
  tag: 'app-settings-general',
  styleUrl: 'app-settings-general.css',
  scoped: true,
})
export class AppSettingsGeneral {
  @State() config: Record<string, unknown>

  private updateSetting = (key: UnprotectedStoreKeys) => (e: Event) => {
    e.preventDefault()
    const target = e.target as HTMLInputElement
    const value = target.checked ?? target.value

    client.config.ui.set({ key, value })
  }

  async componentWillLoad() {
    return client.config.ui.getAll().then(({ value }) => {
      this.config = value
    })
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
              onChange={this.updateSetting(UnprotectedStoreKeys.REPORT_ERRORS)}
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
