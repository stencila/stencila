import { Component, h } from '@stencil/core'
import { href } from '@stencil/router'
import { i18n } from '../../../../i18n'
import { getAvailablePlugins } from '../../app-settings/app-settings-plugins/pluginStore'
import { OnboardingRouter } from '../onboardingRouter'

const recommendedPlugins = ['encoda']

@Component({
  tag: 'app-onboarding-plugins',
  styleUrl: 'app-onboarding-plugins.css',
  scoped: false,
})
export class AppOnboardingPlugins {
  async componentWillLoad() {
    return getAvailablePlugins(recommendedPlugins)
  }

  render() {
    return (
      <div class="app-onboarding">
        <stencila-icon icon="plug"></stencila-icon>

        <h1>{i18n.t('onboarding.plugins.title')}</h1>

        <p>{i18n.t('onboarding.plugins.explanation')}</p>

        {recommendedPlugins.map((pluginName) => (
          <app-settings-plugin-card
            pluginName={pluginName}
          ></app-settings-plugin-card>
        ))}

        <stencila-button {...href('/onboarding/reporting', OnboardingRouter)}>
          {i18n.t('onboarding.plugins.next')}
        </stencila-button>
      </div>
    )
  }
}
