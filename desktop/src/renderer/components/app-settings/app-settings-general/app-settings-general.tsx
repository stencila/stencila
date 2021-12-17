import { Component, h, State } from '@stencil/core'
import { i18n } from '../../../../i18n'
import { GlobalConfigKeys } from '../../../../preload/stores'
import { CombinedConfig, ConfigPaths } from '../../../../preload/types'
import { client } from '../../../client'
import { showAndCaptureError } from '../../../utils/errors'

@Component({
  tag: 'app-settings-general',
  styleUrl: 'app-settings-general.css',
  scoped: true,
})
export class AppSettingsGeneral {
  @State() config: CombinedConfig

  private updateSetting = (key: ConfigPaths) => (e: Event) => {
    e.preventDefault()
    const target = e.target as HTMLInputElement
    const value = target.checked ?? target.value

    client.config
      .set({ key, value: value.toString() })
      .catch((err) => showAndCaptureError(err))
  }

  async componentWillLoad() {
    return client.config.getAll().then(({ value: config }) => {
      this.config = config
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
              defaultChecked={
                this.config.global.telemetry?.desktop?.error_reports ?? false
              }
              // @ts-expect-error
              onChange={this.updateSetting(GlobalConfigKeys.REPORT_ERRORS)}
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
