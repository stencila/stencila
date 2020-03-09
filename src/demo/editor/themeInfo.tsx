import React from 'react'
import { ThemeNames, themes } from '../../browser'
import themeInfoJSON from '../../themes.json'

const themeInfo = themeInfoJSON

interface Props {
  activeTheme: string
}

const isThemeName = (name: string): name is ThemeNames => {
  return Object.keys(themes).includes(name)
}

export const ThemeInfo = ({ activeTheme }: Props): JSX.Element | null => {
  return isThemeName(activeTheme) && themeInfo[activeTheme] !== null ? (
    <p
      className="themeInfo"
      dangerouslySetInnerHTML={{ __html: themeInfo[activeTheme].text }}
    ></p>
  ) : null
}
