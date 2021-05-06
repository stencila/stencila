import { Component, h, State } from '@stencil/core'
import { CHANNEL } from '../../../../preload/index'
import { build, ConfigSchema } from '../../forms/formBuilder'

type Config = Record<string, unknown>
type Settings = {
  config: Config
  schema: ConfigSchema
}

@Component({
  tag: 'app-settings',
  styleUrl: 'app-settings.css',
  scoped: true,
})
export class AppSettings {
  @State() settings: Settings | undefined

  private readConfig = () =>
    (window.api.invoke(CHANNEL.READ_CONFIG) as unknown) as Promise<Settings>

  async componentWillLoad() {
    this.settings = await this.readConfig()
  }

  render() {
    return (
      <div class="app-settings">
        <app-side-nav></app-side-nav>
        <main>
          <form>{this.settings && build(this.settings.schema)}</form>
        </main>
      </div>
    )
  }
}
