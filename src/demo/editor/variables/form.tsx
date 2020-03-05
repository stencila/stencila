import React from 'react'
import { getCssVariables } from '../../parseCss'
import { upsertThemeOverrides } from '../../utils/theme'
import { parseQueries, removeQuery, upsertQuery } from '../../utils/url'
import { VariableInput } from './input'
import { ThemeObject } from '../../utils'

interface Props {
  theme: string
  onContribute: () => void
}

type ThemeSettings = Record<string, ReturnType<typeof getCssVariables>>

const getThemeCSS = (theme: string): string => {
  const req = new XMLHttpRequest()
  req.open('GET', `themes/${theme}/styles.css`, false)
  req.send(null)
  return req.responseText
}

// Build up a form label/input pairs for each variable
const formEls = (
  theme: ThemeObject,
  onChange: (variable: string, value: string, commit?: boolean) => void,
  userTheme: ThemeObject
): JSX.Element[] =>
  Object.entries(theme)
    .sort()
    .reduce(
      (els: JSX.Element[], [name, value]) => [
        ...els,
        <VariableInput
          key={name}
          name={name}
          onChange={onChange}
          value={value}
          valueOverride={userTheme[name]}
        />
      ],
      []
    )

export const VariableKnobs = ({ theme, onContribute }: Props): JSX.Element => {
  const [themeVars, updateThemeVars] = React.useState<ThemeSettings>({})

  const [userVars, updateUserVars] = React.useState<ThemeSettings>({})

  React.useEffect(() => {
    const css = getThemeCSS(theme)
    const variables = getCssVariables(css)
    updateThemeVars({ ...themeVars, [theme]: variables })
    updateUserVars({
      ...userVars,
      [theme]: parseQueries(Object.keys(variables))
    })
  }, [theme])

  React.useEffect(() => {
    upsertThemeOverrides(themeVars[theme] ?? {}, userVars[theme] ?? {})
  }, [userVars])

  const updateVar = (variable: string, value: string, commit = false): void => {
    const customizations = { ...(userVars[theme] ?? {}) }

    if (
      (themeVars[theme] !== undefined &&
        themeVars[theme][variable]?.toLowerCase() === value.toLowerCase()) ||
      value === ''
    ) {
      delete customizations[variable]
      if (commit === true) {
        removeQuery(variable)
      }
    } else {
      customizations[variable] = value
      if (commit === true) {
        upsertQuery(variable, value)
      }
    }

    updateUserVars({
      ...userVars,
      [theme]: customizations
    })

    upsertThemeOverrides(themeVars[theme], customizations)
  }

  const buildForm = React.useCallback(
    () => formEls(themeVars[theme] ?? {}, updateVar, userVars[theme] ?? {}),
    [themeVars, userVars]
  )

  return (
    <div id="themeVariables">
      <form name="themeBuilder">
        {buildForm().length === 0 ? (
          <label>No variables exposed</label>
        ) : (
          buildForm()
        )}
      </form>

      {userVars[theme] !== undefined &&
        Object.keys(userVars[theme]).length > 0 && (
          <>
            <a
              className="button"
              href="#"
              id="contribute"
              target="_blank"
              title="Submit your changes as a new theme to Thema"
              onClick={e => {
                e.preventDefault()
                onContribute()
              }}
            >
              Contribute changes
            </a>
            <div id="contributeCover"></div>
          </>
        )}
    </div>
  )
}
