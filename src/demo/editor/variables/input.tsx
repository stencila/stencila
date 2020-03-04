import React from 'react'
import { ColorInput } from './colorInput'

interface Props {
  name: string
  value: string
  valueOverride?: string
  onChange: (variable: string, value: string, commit?: boolean) => void
}

export const VariableInput = ({
  name,
  value,
  valueOverride,
  onChange
}: Props): JSX.Element => {
  const clear = React.useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault()
      onChange(name, value, true)
    },
    [name]
  )

  return React.useMemo(() => {
    const _value = valueOverride ?? value

    let input: JSX.Element

    if (value.includes('#')) {
      input = <ColorInput name={name} value={_value} onChange={onChange} />
    } else {
      input = (
        <input
          onChange={e => {
            onChange(name, e.currentTarget.value, true)
          }}
          style={name.includes('font-family') ? { fontFamily: _value } : {}}
          defaultValue={_value}
          name={name}
          id={name}
        />
      )
    }

    return (
      <fieldset>
        <div className="labelWrapper">
          <label
            htmlFor={name}
            className={valueOverride !== undefined ? 'modified' : ''}
            title={valueOverride !== undefined ? 'Value has been modified' : ''}
          >
            {name}
          </label>
          {valueOverride !== undefined && (
            <button onClick={clear} type="reset">
              Clear
            </button>
          )}
        </div>

        {input}
      </fieldset>
    )
  }, [name, value, valueOverride])
}
