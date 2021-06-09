import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { OnboardingRouter } from '../onboardingRouter'

@Component({
  tag: 'app-onboarding-root',
  styleUrl: 'app-onboarding-root.css',
  scoped: false,
})
export class AppOnboardingRoot {
  render() {
    return (
      <div class="app-onboarding-root">
        <OnboardingRouter.Switch>
          <Route path="/onboarding">
            <app-onboarding-welcome></app-onboarding-welcome>
          </Route>

          <Route path="/onboarding/plugins">
            <app-onboarding-plugins></app-onboarding-plugins>
          </Route>

          <Route path="/onboarding/reporting">
            <app-onboarding-reporting></app-onboarding-reporting>
          </Route>

          <Route path="/onboarding/end">
            <app-onboarding-end></app-onboarding-end>
          </Route>
        </OnboardingRouter.Switch>
      </div>
    )
  }
}
