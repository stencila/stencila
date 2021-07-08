import { Component, h } from '@stencil/core'
import { href } from '@stencil/router'
import { i18n } from '../../../../i18n'
import { CHANNEL } from '../../../../preload/channels'
import { UnprotectedStoreKeys } from '../../../../preload/stores'
import { OnboardingRouter } from '../onboardingRouter'

@Component({
  tag: 'app-onboarding-reporting',
  styleUrl: 'app-onboarding-reporting.css',
  scoped: false,
})
export class AppOnboardingReoporting {
  private enableReporting = (e: MouseEvent) => {
    e.preventDefault()
    window.api
      .invoke(CHANNEL.SET_APP_CONFIG, {
        key: UnprotectedStoreKeys.REPORT_ERRORS,
        value: true,
      })
      .then(() => {
        OnboardingRouter.push('/onboarding/end')
      })
  }

  render() {
    return (
      <div class="app-onboarding">
        <stencila-icon icon="heart-pulse"></stencila-icon>

        <h1>{i18n.t('settings.general.crashReports.label')}</h1>

        <p>{i18n.t('settings.general.crashReports.help')}</p>

        <stencila-button onClick={this.enableReporting}>
          {i18n.t('onboarding.reporting.confirm')}
        </stencila-button>

        <a {...href('/onboarding/end', OnboardingRouter)} class="skipSection">
          {i18n.t('onboarding.reporting.next')}
        </a>
      </div>
    )
  }
}
