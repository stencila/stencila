import { FunctionalComponent, h } from '@stencil/core'
import { CHANNEL } from '../../../../../preload/index'
import { plugins } from 'stencila'

type Props = Pick<plugins.Plugin, 'name' | 'next' | 'installation'>

const install = (name: string) =>
  (window.api.invoke(CHANNEL.INSTALL_PLUGIN, name) as unknown) as Promise<
    Plugin[]
  >

const uninstall = (name: string) =>
  (window.api.invoke(CHANNEL.UNINSTALL_PLUGIN, name) as unknown) as Promise<
    Plugin[]
  >

export const PluginInstallButton: FunctionalComponent<Props> = (props) => {
  if (props.next) {
    return (
      <stencila-button icon="refresh" size="small">
        Upgrade
      </stencila-button>
    )
  }

  if (props.installation) {
    return (
      <stencila-button
        icon="delete-bin-2"
        onClick={() => uninstall(props.name)}
        size="small"
      >
        Uninstall
      </stencila-button>
    )
  }

  return (
    <stencila-button
      icon="download"
      onClick={() => install(props.name)}
      size="small"
    >
      Install
    </stencila-button>
  )
}
