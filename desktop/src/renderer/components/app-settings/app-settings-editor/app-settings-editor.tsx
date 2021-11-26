import { Component, h, State } from '@stencil/core'
import { FileFormatUtils } from '@stencila/components'
import { Config } from 'stencila'
import { i18n } from '../../../../i18n'
import { GlobalConfigKeys } from '../../../../preload/stores'
import { AppConfigStore, ConfigPaths } from '../../../../preload/types'
import { client } from '../../../client'

@Component({
  tag: 'app-settings-editor',
  styleUrl: 'app-settings-editor.css',
  scoped: true,
})
export class AppSettingsEditor {
  @State() config: {
    app: AppConfigStore
    global: Config
  }

  private updateSetting = (key: ConfigPaths) => (e: Event) => {
    e.preventDefault()
    const target = e.target as HTMLInputElement
    const value = target.checked ?? target.value

    client.config.set({ key, value: value.toString() })
  }

  async componentWillLoad() {
    return client.config.getAll().then(({ value: config }) => {
      this.config = config
    })
  }

  render() {
    return (
      <div class="appSettingsEditor">
        <div class="title">
          <h1>{i18n.t('settings.editor.title')}</h1>
        </div>

        <form>
          <fieldset>
            <label htmlFor="newFileDefault">
              {i18n.t('settings.editor.newFileDefault.label')}
            </label>

            <select
              id="newFileDefault"
              onChange={this.updateSetting(
                GlobalConfigKeys.EDITOR_NEW_FILE_SYNTAX
              )}
            >
              {Object.values(FileFormatUtils.fileFormatMap).map((format) => (
                <option
                  value={format.ext ?? format.name}
                  selected={
                    this.config.global.editors?.defaultFormat === format.ext ||
                    this.config.global.editors?.defaultFormat === format.name
                  }
                  key={format.name}
                >
                  {format.name}
                </option>
              ))}
            </select>

            <p class="helpText">
              {i18n.t('settings.editor.newFileDefault.help')}
            </p>
          </fieldset>

          <fieldset>
            <input
              id="lineWrap"
              type="checkbox"
              defaultChecked={this.config.global.editors?.lineWrapping ?? false}
              onChange={this.updateSetting(
                GlobalConfigKeys.EDITOR_LINE_WRAPPING
              )}
            />

            <label htmlFor="lineWrap">
              {i18n.t('settings.editor.lineWrap.label')}
            </label>
          </fieldset>

          <fieldset>
            <input
              id="lineNumbers"
              type="checkbox"
              defaultChecked={this.config.global.editors?.lineNumbers ?? false}
              onChange={this.updateSetting(
                GlobalConfigKeys.EDITOR_LINE_NUMBERS
              )}
            />

            <label htmlFor="lineNumbers">
              {i18n.t('settings.editor.lineNumbers.label')}
            </label>
          </fieldset>
        </form>
      </div>
    )
  }
}
