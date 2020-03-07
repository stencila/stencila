import React from 'react'
import { getTheme } from '../utils/theme'
import { Header } from './header'
import { ThemeVariables } from './theme'
import { ThemeInfo } from './themeInfo'
import { ThemeSwitcher } from './themeSwitcher'

type Props = {}

interface State {
  activeTheme: string
}

export class ThemeEditor extends React.PureComponent<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = {
      activeTheme: getTheme()
    }
  }

  setTheme = (theme: string): void => {
    this.setState({ activeTheme: theme })
  }

  render(): JSX.Element {
    return (
      <>
        <Header />

        <h2 id="themeName">
          <span>Theme</span>

          <ThemeSwitcher
            activeTheme={this.state.activeTheme}
            onChangeTheme={this.setTheme}
          />
        </h2>

        <ThemeInfo activeTheme={this.state.activeTheme} />

        <h3>Customize</h3>

        <ThemeVariables activeTheme={this.state.activeTheme} />
      </>
    )
  }
}
