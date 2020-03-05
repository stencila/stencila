// @ts-ignore
import generate from 'project-name-generator'
import React from 'react'
import ReactDOM from 'react-dom'
import { submitPR, ThemeObject } from '../utils'

const randomName = (): string => generate().dashed

interface Props {
  baseTheme: ThemeObject
  baseThemeName: string
  themeOverrides: ThemeObject
  onClose: () => void
}

interface State {
  projectName: string
}

export class ContributeForm extends React.Component<Props, State> {
  private el: HTMLElement | null

  constructor(props: Props) {
    super(props)

    this.el = document.getElementById('modalTarget')

    this.state = {
      projectName: randomName()
    }
  }

  setProjectName = (projectName: string): void => {
    this.setState({ projectName })
    this.el = document.getElementById('modal')
  }

  onNameChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    e.preventDefault()
    this.setProjectName(e.currentTarget.value)
  }

  onRandomizeName = (e: React.MouseEvent<HTMLButtonElement>): void => {
    e.preventDefault()
    this.setProjectName(randomName())
  }

  onSubmitTheme = (e: React.MouseEvent<HTMLButtonElement>): void => {
    e.preventDefault()

    submitPR(
      this.state.projectName,
      '',
      this.props.themeOverrides,
      this.props.baseThemeName,
      this.props.baseTheme
    )
  }

  render(): React.ReactPortal | null {
    return this.el === null
      ? null
      : ReactDOM.createPortal(
          <div id="contributeModal" onClick={this.props.onClose}>
            <div className="modalContents" onClick={e => e.stopPropagation()}>
              <p>
                Name your theme, and submit as a GitHub pull request to share
                your theme with others.
              </p>

              <form>
                <label>Theme Name</label>

                <div className="labelWrapper">
                  <input
                    value={this.state.projectName}
                    onChange={this.onNameChange}
                  />

                  <button type="button" onClick={this.onRandomizeName}>
                    Randomize
                  </button>
                </div>

                <button onClick={this.onSubmitTheme} type="submit">
                  Open Pull Request
                </button>
              </form>
            </div>
          </div>,
          this.el
        )
  }
}
