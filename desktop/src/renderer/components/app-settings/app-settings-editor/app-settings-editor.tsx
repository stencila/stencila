import { Component, h, State } from '@stencil/core'
import { i18n } from '../../../../i18n'
import { UnprotectedStoreKeys } from '../../../../preload/stores'
import { AppConfigStore } from '../../../../preload/types'
import { client } from '../../../client'

@Component({
  tag: 'app-settings-editor',
  styleUrl: 'app-settings-editor.css',
  scoped: true,
})
export class AppSettingsEditor {
  @State() config: AppConfigStore

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
                UnprotectedStoreKeys.EDITOR_NEW_FILE_SYNTAX
              )}
            >
              <option
                value="md"
                selected={this.config.EDITOR_NEW_FILE_SYNTAX === 'md'}
              >
                Markdown
              </option>
              <option
                value="rmd"
                selected={this.config.EDITOR_NEW_FILE_SYNTAX === 'rmd'}
              >
                RMD
              </option>
              <option
                value="r"
                selected={this.config.EDITOR_NEW_FILE_SYNTAX === 'r'}
              >
                R
              </option>
            </select>

            <p class="helpText">
              {i18n.t('settings.editor.newFileDefault.help')}
            </p>
          </fieldset>

          <fieldset>
            <input
              id="lineWrap"
              type="checkbox"
              defaultChecked={this.config.EDITOR_LINE_WRAPPING ?? false}
              onChange={this.updateSetting(
                UnprotectedStoreKeys.EDITOR_LINE_WRAPPING
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
              defaultChecked={this.config.EDITOR_LINE_NUMBERS ?? false}
              onChange={this.updateSetting(
                UnprotectedStoreKeys.EDITOR_LINE_NUMBERS
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
