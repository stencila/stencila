import { Component, h } from '@stencil/core'
import { href } from '@stencil/router'
import { i18n } from '../../../../i18n'
import { SettingsRouter } from '../settingsRouter'

@Component({
  tag: 'app-settings-sidebar',
  styleUrl: 'app-settings-sidebar.css',
  scoped: true,
})
export class AppSettingsSidebar {
  render() {
    const activePath = SettingsRouter.path

    return (
      <nav class="app-side-nav">
        <ul>
          <li>
            <a
              {...href('/settings')}
              class={{ active: activePath === '/settings', disabled: true }}
            >
              <stencila-icon icon="settings-3"></stencila-icon>
              <span>{i18n.t('settings.general.title')}</span>
            </a>
          </li>

          <li>
            <a
              {...href('/settings/advanced')}
              class={{
                navItem: true,
                active: activePath === '/settings/advanced',
              }}
            >
              <stencila-icon icon="list-settings"></stencila-icon>
              <span>Advanced</span>
            </a>
          </li>

          <li>
            <a {...href('/settings/')} class={{ disabled: true }}>
              <stencila-icon icon="user"></stencila-icon>
              Account
            </a>
          </li>

          <li>
            <a {...href('/settings/')} class={{ disabled: true }}>
              <stencila-icon icon="palette"></stencila-icon>
              Appearance
            </a>
          </li>

          <li>
            <a
              {...href('/settings/plugins')}
              class={{
                navItem: true,
                active: activePath === '/settings/plugins',
              }}
            >
              <stencila-icon icon="plug"></stencila-icon>
              {i18n.t('settings.plugins.title')}
            </a>
          </li>

          <li>
            <a {...href('/settings/')} class={{ disabled: true }}>
              <stencila-icon icon="file-edit"></stencila-icon>
              Editor
            </a>
          </li>

          <li>
            <a {...href('/settings/')} class={{ disabled: true }}>
              <stencila-icon icon="newspaper"></stencila-icon>
              Publishing
            </a>
          </li>
        </ul>
      </nav>
    )
  }
}
