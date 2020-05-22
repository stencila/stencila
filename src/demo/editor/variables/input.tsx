import React from 'react'
import { ColorInput } from './colorInput'

interface Props {
  name: string
  value: string
  valueOverride?: string
  onChange: (variable: string, value: string, commit?: boolean) => void
}

interface State {
  hasError: boolean
}

export class VariableInput extends React.PureComponent<Props, {}> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(): State {
    return { hasError: true }
  }

  componentDidCatch(error: unknown, errorInfo: unknown): void {
    console.error(error, errorInfo)
  }

  clear = (e: React.MouseEvent<HTMLButtonElement>): void => {
    e.preventDefault()
    this.props.onChange(this.props.name, this.props.value, true)
  }

  onChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    e.preventDefault()
    this.props.onChange(this.props.name, e.currentTarget.value, true)
  }

  render(): JSX.Element {
    const _value = this.props.valueOverride ?? this.props.value

    let input: JSX.Element

    if (this.props.name.includes('color') && !_value.includes('var')) {
      input = (
        <ColorInput
          name={this.props.name}
          value={_value}
          onChange={this.props.onChange}
        />
      )
    } else {
      input = (
        <input
          onChange={this.onChange}
          style={
            this.props.name.includes('font-family')
              ? { fontFamily: _value }
              : {}
          }
          defaultValue={_value}
          name={this.props.name}
          id={this.props.name}
        />
      )
    }

    return (
      <fieldset>
        <div className="fieldsetWrapper">
          <div className="labelWrapper">
            <label
              htmlFor={this.props.name}
              className={
                this.props.valueOverride === undefined ? '' : 'modified'
              }
              title={
                this.props.valueOverride === undefined
                  ? ''
                  : 'Value has been modified'
              }
            >
              {this.props.name}
            </label>
            {this.props.valueOverride !== undefined && (
              <button onClick={this.clear} type="reset">
                Clear
              </button>
            )}
          </div>

          {input}
        </div>
      </fieldset>
    )
  }
}
