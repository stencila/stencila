import { Component, h } from '@stencil/core'

@Component({
  tag: 'app-side-nav',
  styleUrl: 'app-side-nav.css',
  scoped: true,
})
export class AppSideNav {
  render() {
    return (
      <nav class="app-side-nav">
        <ul>
          <li>
            <a href="#" class="nav-item active">
              <stencila-icon icon="settings-2"></stencila-icon>
              <span>General</span>
            </a>

            <ul>
              <li>
                <a href="#" class="nav-item">
                  <span>Advanced</span>
                </a>
              </li>
            </ul>
          </li>

          <li>
            <a href="#" class="nav-item disabled">
              <stencila-icon icon="user"></stencila-icon>
              Account
            </a>
          </li>

          <li>
            <a href="#" class="nav-item disabled">
              <stencila-icon icon="palette"></stencila-icon>
              Appearances
            </a>
          </li>

          <li>
            <a href="#" class="nav-item disabled">
              <stencila-icon icon="plug"></stencila-icon>
              Plugins
            </a>
          </li>

          <li>
            <a href="#" class="nav-item disabled">
              <stencila-icon icon="file-edit"></stencila-icon>
              Editors
            </a>
          </li>

          <li>
            <a href="#" class="nav-item disabled">
              <stencila-icon icon="newspaper"></stencila-icon>
              Publishing
            </a>
          </li>
        </ul>
      </nav>
    )
  }
}
