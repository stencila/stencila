import { Component, h } from '@stencil/core'
import { href } from '@stencil/router'
import { i18n } from '../../../../i18n'
import { getAvailablePlugins } from '../../app-settings/app-settings-plugins/pluginStore'

@Component({
  tag: 'app-onboarding-plugins',
  styleUrl: 'app-onboarding-plugins.css',
  scoped: false,
})
export class AppOnboardingPlugins {
  async componentWillLoad() {
    return getAvailablePlugins()
  }

  render() {
    return (
      <div class="app-onboarding">
        <stencila-icon icon="plug"></stencila-icon>

        <h1>{i18n.t('onboarding.plugins.title')}</h1>

        <p>{i18n.t('onboarding.plugins.explanation')}</p>

        <app-settings-plugin-card pluginName="encoda"></app-settings-plugin-card>

        <stencila-button {...href('/onboarding/reporting')}>
          {i18n.t('onboarding.plugins.next')}
        </stencila-button>
      </div>
    )
  }
}
