import React, { useState } from 'react'
import { getTheme } from '../utils/theme'
import { Header } from './header'
import { ThemeSwitcher } from './themeSwitcher'
import { VariableKnobs } from './variables/form'
import { ContributeForm } from './contributeModal'
import { ThemeObject } from '../utils'

const theme = getTheme()

export const ThemeEditor = (): JSX.Element => {
  const [activeTheme, setTheme] = useState<string>(theme)
  const [contributeModalIsOpen, setContributeModal] = useState<boolean>(false)
  const [themeOverrides, setThemeOverrides] = useState<
    Record<string, ThemeObject>
  >({})

  const openContributeModal = React.useCallback(
    () => setContributeModal(true),
    []
  )

  const closeContributeModal = React.useCallback(
    () => setContributeModal(false),
    []
  )

  console.log('rendering editor')

  return (
    <>
      <Header />

      <h2 id="themeName">
        <span>Theme</span>

        <ThemeSwitcher activeTheme={activeTheme} onChangeTheme={setTheme} />
      </h2>

      <hr />

      <h3>Customize</h3>

      <VariableKnobs theme={activeTheme} onContribute={openContributeModal} />

      {contributeModalIsOpen && (
        <ContributeForm
          themeOverrides={themeOverrides[activeTheme] ?? {}}
          baseTheme={themeOverrides[activeTheme] ?? {}}
          baseThemeName={activeTheme}
          onClose={closeContributeModal}
        />
      )}
    </>
  )
}
