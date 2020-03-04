import React from 'react'
import { getCssVariables } from '../../parseCss'
import { upsertThemeOverrides } from '../../utils/theme'
import { parseQueries, removeQuery, upsertQuery } from '../../utils/url'
import { VariableInput } from './input'
import { submitPR } from '../../utils'

interface Props {
  theme: string
}

type ThemeSettings = Record<string, ReturnType<typeof getCssVariables>>

const getThemeCSS = (theme: string): string => {
  const req = new XMLHttpRequest()
  req.open('GET', `themes/${theme}/styles.css`, false)
  req.send(null)
  return req.responseText
}

export const VariableKnobs = ({ theme }: Props): JSX.Element => {
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

  const onChange = React.useCallback(updateVar, [themeVars, userVars])

  // Build up a form label/input pairs for each variable
  const formEls = Object.entries(themeVars[theme] ?? {})
    .sort()
    .reduce(
      (els: JSX.Element[], [name, value]) => [
        ...els,
        <VariableInput
          key={name}
          name={name}
          onChange={onChange}
          value={value}
          valueOverride={userVars[theme][name]}
        />
      ],
      []
    )

  return (
    <div id="themeVariables">
      <form name="themeBuilder">
        {formEls.length === 0 ? <label>No variables exposed</label> : formEls}
      </form>

      {userVars[theme] !== undefined &&
        Object.keys(userVars[theme]).length > 0 && (
          <a
            className="button"
            href="#"
            id="contribute"
            target="_blank"
            title="Submit your changes as a new theme to Thema"
            onClick={e => {
              e.preventDefault()
              submitPR('', '', userVars[theme], theme, themeVars[theme])
            }}
          >
            Contribute changes
          </a>
        )}
    </div>
  )
}
