import React from 'react'
import '@simonwep/pickr/dist/themes/monolith.min.css' // 'monolith' theme

import Pickr from '@simonwep/pickr'

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
          this.props.onChange(this.props.name, colorValue(color), true)
          instance.hide()
        })
        .on('hide', (instance: Pickr) => {
          this.props.onChange(
            this.props.name,
            colorValue(instance.getColor()),
            true
          )
        })
        .on('cancel', (instance: Pickr) => {
          instance.setColor(this.originalColor)
          this.props.onChange(this.props.name, this.originalColor)
          instance.hide()
        })
        .on('changestop', (instance: Pickr) => {
          this.props.onChange(this.props.name, colorValue(instance.getColor()))
        })
        .on('swatchselect', (color: Pickr.HSVaColor) => {
          this.props.onChange(this.props.name, colorValue(color))
        })

      this.setState({
        pickr
      })
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
        <button ref={this.pickrEl} />
        <input
          value={this.props.value}
          name={this.props.name}
          id={this.props.name}
          onChange={e => {
            e.preventDefault()
            this.props.onChange(this.props.name, e.currentTarget.value)
          }}
          onFocus={() => {
            if (this.state.pickr !== null) {
              this.state.pickr.show()
            }
          }}
        />
      </>
    )
  }
}
