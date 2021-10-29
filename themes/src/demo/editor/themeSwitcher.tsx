import React from 'react'
import { themes } from '../../themes'
import { themeSet } from '../utils/theme'
import { getPreview } from '../utils/preview'

interface Props {
  activeTheme: string
  onChangeTheme: (theme: string) => void
}

export class ThemeSwitcher extends React.Component<Props> {
  previewEl: HTMLIFrameElement | null = null
  poll: number | undefined

  componentDidMount(): void {
    this.previewEl = getPreview()

    if (this.previewEl !== null) {
      this.previewEl.addEventListener('load', () => {
        themeSet(this.props.activeTheme)
      })
    } else {
      this.poll = window.setInterval(() => {
        this.previewEl = getPreview()
        if (this.previewEl !== null) {
          window.clearInterval(this.poll)
          themeSet(this.props.activeTheme)
        }
      }, 300)
    }
  }

  componentWillUnmount(): void {
    window.clearInterval(this.poll)
  }

  render(): JSX.Element {
    return (
      <select
        defaultValue={this.props.activeTheme}
        onChange={e => {
          e.preventDefault()
          const theme = e.currentTarget.value
          this.props.onChangeTheme(theme)
          themeSet(theme)
        }}
      >
        {Object.entries(themes).map(([path, name]) => (
          <option key={path} value={name}>
            {name}
          </option>
        ))}
      </select>
    )
  }
}
