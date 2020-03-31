import Pickr from '@simonwep/pickr'
import '@simonwep/pickr/dist/themes/monolith.min.css' // 'monolith' theme
import React from 'react'

interface Props {
  name: string
  value: string
  onChange: (variable: string, value: string, commit?: boolean) => void
}

interface State {
  pickr: Pickr | null
}

type Snapshot = null | {
  valueChanged: boolean
}

export class ColorInput extends React.PureComponent<Props, State> {
  pickrEl: React.MutableRefObject<HTMLButtonElement | null>
  originalColor: Props['value']

  constructor(props: Props) {
    super(props)

    this.pickrEl = React.createRef<HTMLButtonElement | null>()
    this.originalColor = props.value

    this.state = {
      pickr: null
    }
  }

  setColor = (value: string, commit = false): void => {
    this.props.onChange(this.props.name, value, commit)
  }

  setPickrColor = (value: string): void => {
    if (this.state.pickr !== null) {
      this.state.pickr.setColor(value)
    }
  }

  showPickr = (): void => {
    if (this.state.pickr !== null) {
      this.state.pickr.show()
    }
  }

  hidePickr = (): void => {
    if (this.state.pickr !== null) {
      this.state.pickr.hide()
    }
  }

  onChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    e.preventDefault()
    this.setColor(e.currentTarget.value)
  }

  onBlur = (e: React.ChangeEvent<HTMLInputElement>): void => {
    this.setPickrColor(e.currentTarget.value)
    this.setColor(e.currentTarget.value)
  }

  onKeyUp = (e: React.KeyboardEvent<HTMLInputElement>): void => {
    if (e.key === 'Enter') {
      this.setPickrColor(e.currentTarget.value)
      this.setColor(e.currentTarget.value)
    }
  }

  componentDidMount(): void {
    if (this.pickrEl.current !== null) {
      const pickr = Pickr.create({
        el: this.pickrEl.current,
        default: this.props.value,
        theme: 'monolith',
        comparison: false, // This allows the color swatch to update immediately

        swatches: [
          'rgba(244, 67, 54, 1)',
          'rgba(233, 30, 99, 1)',
          'rgba(156, 39, 176, 1)',
          'rgba(103, 58, 183, 1)',
          'rgba(63, 81, 181, 1)',
          'rgba(33, 150, 243, 1)',
          'rgba(3, 169, 244, 1)',
          'rgba(0, 188, 212, 1)',
          'rgba(0, 150, 136, 1)',
          'rgba(76, 175, 80, 1)',
          'rgba(139, 195, 74, 1)',
          'rgba(205, 220, 57, 1)',
          'rgba(255, 235, 59, 1)',
          'rgba(255, 193, 7, 1)'
        ],

        components: {
          // Main components
          preview: true,
          opacity: true,
          hue: true,

          // Input / output Options
          interaction: {
            hex: true,
            rgba: true,
            hsla: false,
            hsva: false,
            cmyk: false,
            input: true,
            clear: false,
            cancel: true,
            save: true
          }
        }
      })

      const colorValue = (color: Pickr.HSVaColor): string => {
        // @ts-ignore
        return color[`to${pickr.getColorRepresentation()}`]().toString(0)
      }

      pickr
        .on('show', () => {
          this.originalColor = this.props.value
        })
        .on('save', (color: Pickr.HSVaColor, instance: Pickr) => {
          this.setColor(colorValue(color), true)
          instance.hide()
        })
        .on('hide', (instance: Pickr) => {
          this.setColor(this.props.value, true)
        })
        .on('cancel', (instance: Pickr) => {
          instance.setColor(this.originalColor)
          this.setColor(this.originalColor)
          instance.hide()
        })
        // TODO: Evaluate performance of live color updates
        // .on('change', (color: Pickr.HSVaColor) => {
        //   this.setColor(colorValue(color))
        // })
        .on('changestop', (instance: Pickr) => {
          this.setColor(colorValue(instance.getColor()))
        })
        .on('swatchselect', (color: Pickr.HSVaColor) => {
          this.setColor(colorValue(color))
        })

      this.setState({
        pickr
      })

      // Remove keyboard focus from the Swatch button, as the input field opens the swatch
      // @ts-ignore
      const pickrEl: HTMLDivElement = pickr.getRoot().button
      pickrEl.setAttribute('tabIndex', '-1')
    }
  }

  getSnapshotBeforeUpdate(prevProps: Props): Snapshot {
    if (
      this.props.value !== prevProps.value &&
      this.state.pickr !== null &&
      !this.state.pickr.isOpen()
    ) {
      return { valueChanged: true }
    }

    return null
  }

  componentDidUpdate(
    _prevProps: Props,
    _prevState: State,
    snapshot: Snapshot
  ): void {
    if (
      snapshot !== null &&
      snapshot.valueChanged === true &&
      this.state.pickr !== null
    ) {
      this.state.pickr.setColor(this.props.value)
    }
  }

  componentWillUnmount(): void {
    if (this.state.pickr !== null) {
      this.state.pickr.destroyAndRemove()
    }
  }

  render(): JSX.Element {
    return (
      <>
        <button ref={this.pickrEl} tabIndex={-1} />
        <input
          id={this.props.name}
          name={this.props.name}
          onBlur={this.onBlur}
          onChange={this.onChange}
          onFocus={this.showPickr}
          onKeyUp={this.onKeyUp}
          value={this.props.value}
        />
      </>
    )
  }
}
