import { Component, h } from '@stencil/core'
import { i18n } from '../../../../i18n'
import { captureError } from '../../../../preload/errors'
import { client } from '../../../client'
import { showAndCaptureError } from '../../../utils/errors'

@Component({
  tag: 'app-onboarding-end',
  styleUrl: 'app-onboarding-end.css',
  scoped: false,
})
export class AppOnboardingEnd {
  private openLinkInBrowser = (url: string) => (e: MouseEvent) => {
    e.preventDefault()
    client.app.utils
      .openLinkInBrowser(url)
      .catch((err) => showAndCaptureError(err))
  }

  private nextHandler = () => {
    client.launcher.open().finally(() => {
      client.onboarding.close().catch((err) => {
        captureError(err)
      })
    })
  }

  render() {
    return (
      <div class="app-onboarding">
        <stencila-icon icon="rocket"></stencila-icon>

        <h1>{i18n.t('onboarding.end.title')}</h1>

        <p>{i18n.t('onboarding.end.explanation')}</p>

        <p>{i18n.t('onboarding.end.resources.info')}</p>

        <ul>
          <li>
            <a
              href="https://help.stenci.la"
              onClick={this.openLinkInBrowser('https://help.stenci.la')}
            >
              <stencila-icon icon="lifebuoy"></stencila-icon>
              <span>{i18n.t('onboarding.end.resources.help')}</span>
            </a>
          </li>

          <li>
            <a
              href="https://discord.gg/pzUz8R3"
              onClick={this.openLinkInBrowser('https://discord.gg/pzUz8R3')}
            >
              <stencila-icon icon="discord"></stencila-icon>
              <span>{i18n.t('onboarding.end.resources.chat')}</span>
            </a>
          </li>
        </ul>

        <stencila-button onClick={this.nextHandler}>
          {i18n.t('onboarding.end.next')}
        </stencila-button>
      </div>
    )
  }
}
