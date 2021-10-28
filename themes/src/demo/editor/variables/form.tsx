import React from 'react'
import { ThemeObject } from '../../utils'
import { VariableInput } from './input'

interface Props {
  theme: ThemeObject
  themeOverrides: ThemeObject
  onChange: (variable: string, value: string, commit?: boolean) => void
  onContribute: () => void
}

// Build up a form label/input pairs for each variable
const formEls = (
  themeObj: ThemeObject,
  themeOverrides: ThemeObject,
  onChange: (variable: string, value: string, commit?: boolean) => void
): JSX.Element[] =>
  Object.entries(themeObj)
    .sort()
    .reduce(
      (els: JSX.Element[], [name]) => [
        ...els,
        <VariableInput
          key={name}
          name={name}
          onChange={onChange}
          value={themeObj[name]}
          valueOverride={themeOverrides[name]}
        />
      ],
      []
    )

export const VariableKnobs = ({
  theme,
  themeOverrides,
  onChange,
  onContribute
}: Props): JSX.Element => {
  const buildForm = React.useCallback(
    () => formEls(theme, themeOverrides, onChange),
    [theme, themeOverrides]
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

      {Object.keys(themeOverrides).length > 0 && (
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
