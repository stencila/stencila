import React, { useState } from 'react'
import { themes } from '../../themes'
import { Header } from './header'
import { ThemeSwitcher } from './themeSwitcher'
import { VariableKnobs } from './variables/form'

export const ThemeEditor = (): JSX.Element => {
  const [activeTheme, setTheme] = useState<string>(themes.stencila)

  return (
    <>
      <Header />

      <h2 id="themeName">
        <span>Current Theme</span>

        <ThemeSwitcher
          activeTheme={activeTheme}
          onChangeTheme={theme => setTheme(theme)}
        />
      </h2>

      <hr />

      <p>
        Themes are designed to be customizable, if you’d like to make extensive
        changes you can extend a theme, or make one from scratch.{' '}
        <a href="https://github.com/stencila/thema/">Read the documentation</a>{' '}
        to learn how.
      </p>

      <h3>Customize</h3>

      <VariableKnobs theme={activeTheme} />
    </>
  )
}
