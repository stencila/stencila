import React, { useState } from 'react'
import { getTheme } from '../utils/theme'
import { Header } from './header'
import { ThemeSwitcher } from './themeSwitcher'
import { VariableKnobs } from './variables/form'
import { ContributeForm } from './contributeModal'
import { ThemeObject } from '../utils'

export const ThemeEditor = (): JSX.Element => {
  const [activeTheme, setTheme] = useState<string>(getTheme())
  const [contributeModalIsOpen, toggleContributeModal] = useState<boolean>(true)
  const [themeOverrides, setThemeOverrides] = useState<
    Record<string, ThemeObject>
  >({})

  return (
    <>
      <Header />

      <h2 id="themeName">
        <span>Theme</span>

        <ThemeSwitcher
          activeTheme={activeTheme}
          onChangeTheme={theme => setTheme(theme)}
        />
      </h2>

      <hr />

      <h3>Customize</h3>

      <VariableKnobs theme={activeTheme} />

      {contributeModalIsOpen && (
        <ContributeForm
          themeOverrides={themeOverrides[activeTheme] ?? {}}
          baseTheme={themeOverrides[activeTheme] ?? {}}
          baseThemeName={activeTheme}
        />
      )}
    </>
  )
}
