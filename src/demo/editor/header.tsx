import React from 'react'
import ReactDOM from 'react-dom'
import { examples } from '../../examples'
import { getExample, setExample } from '../utils/preview'
import { ViewportToggle } from './viewportToggle'

interface Props {
  exampleContent: string[]
}

export const HeaderBase = (): JSX.Element => {
  return (
    <a href="/">
      <img src="https://stenci.la/img/stencila-logo.svg" />

      <span className="name"> Thema</span>
    </a>
  )
}

const HeaderComponent = ({ exampleContent }: Props): JSX.Element => {
  const [example, updateExample] = React.useState<string>(getExample())

  const onChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    e.preventDefault()
    updateExample(e.currentTarget.value)
    setExample(e.currentTarget.value)
  }

  return (
    <>
      <HeaderBase />

      <div className="tools">
        <ViewportToggle />

        <label>
          Example
          <select
            id="example-select"
            defaultValue={example}
            onChange={onChange}
          >
            {exampleContent.map(ex => (
              <option key={ex}>{ex}</option>
            ))}
          </select>
        </label>

        <a className="github" href="https://github.com/stencila/thema">
          <img src="https://unpkg.com/simple-icons@latest/icons/github.svg" />
        </a>
      </div>
    </>
  )
}

const exampleContent = Object.keys(examples)

export class Header extends React.PureComponent {
  private el: HTMLElement | null

  constructor(props: {}) {
    super(props)
    this.el = document.getElementById('header')
  }

  componentDidMount(): void {
    setExample(getExample())
  }

  render(): React.ReactPortal | null {
    return this.el === null
      ? null
      : ReactDOM.createPortal(
          <HeaderComponent exampleContent={exampleContent} />,
          this.el
        )
  }
}
