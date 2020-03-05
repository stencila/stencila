import React from 'react'
import ReactDOM from 'react-dom'
import { examples } from '../../examples'
import { ViewportToggle } from './viewportToggle'
import { getExample, setExample } from '../utils/preview'
import { HeaderBase } from './headerBase'

interface Props {
  /* onExampleChange: (e: React.ChangeEvent<HTMLSelectElement>) => string */
  exampleContent: string[]
  /* themes: string[] */
}

const HeaderComponent = ({ exampleContent }: Props): JSX.Element => {
  const [example, updateExample] = React.useState<string>(getExample())

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
            onChange={e => {
              e.preventDefault()
              updateExample(e.currentTarget.value)
              setExample(e.currentTarget.value)
            }}
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

export class Header extends React.Component {
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
          <HeaderComponent exampleContent={Object.keys(examples)} />,
          this.el
        )
  }
}
