import React from 'react'
import { getCssVariables } from '../parseCss'
import { ThemeObject } from '../utils'
import { getPreview } from '../utils/preview'
import { getThemeCSS, upsertThemeOverrides } from '../utils/theme'
import { parseQueries, removeQuery, upsertQuery } from '../utils/url'
import { ContributeForm } from './contributeModal'
import { VariableKnobs } from './variables/form'

export type ThemeConfigs = Record<string, ThemeObject>

type Props = {
  activeTheme: string
}

interface State {
  contributeModalIsOpen: boolean
  themeOverrides: ThemeConfigs
  themes: ThemeConfigs
}

export class ThemeVariables extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = {
      contributeModalIsOpen: false,
      themes: {},
      themeOverrides: {}
    }
  }

  setThemeOverrides = (
    variable: string,
    value: string,
    commit = false
  ): void => {
    const customizations = {
      ...(this.state.themeOverrides[this.props.activeTheme] ?? {})
    }

    if (
      (this.state.themes[this.props.activeTheme] !== undefined &&
        this.state.themes[this.props.activeTheme][variable]?.toLowerCase() ===
          value.toLowerCase()) ||
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

    if (
      JSON.stringify(customizations) !==
      JSON.stringify(this.state.themeOverrides[this.props.activeTheme])
    ) {
      upsertThemeOverrides(
        this.state.themes[this.props.activeTheme],
        customizations
      )

      this.setState({
        themeOverrides: {
          ...this.state.themeOverrides,
          [this.props.activeTheme]: customizations
        }
      })
    }
  }

  openContributeModal = (): void =>
    this.setState({ contributeModalIsOpen: true })

  closeContributeModal = (): void =>
    this.setState({ contributeModalIsOpen: false })

  fetchCSS = (useQueryParams = false): void => {
    const css = getThemeCSS(this.props.activeTheme)
    const variables = getCssVariables(css)

    this.setState(
      {
        ...this.state,
        themes: { ...this.state.themes, [this.props.activeTheme]: variables },
        themeOverrides: {
          ...this.state.themeOverrides,
          [this.props.activeTheme]: useQueryParams
            ? parseQueries(Object.keys(variables ?? {}))
            : this.state.themeOverrides[this.props.activeTheme] ?? {}
        }
      },
      this.injectOverrides
    )
  }

  injectOverrides = (): void => {
    upsertThemeOverrides(
      this.state.themes[this.props.activeTheme],
      this.state.themeOverrides[this.props.activeTheme]
    )
  }

  componentDidMount(): void {
    this.fetchCSS(true)

    const preview = getPreview()
    if (preview !== null) {
      preview.addEventListener('load', this.injectOverrides)
    }
  }

  componentDidUpdate(prevProps: Props): void {
    if (
      prevProps.activeTheme !== this.props.activeTheme &&
      this.state.themes[this.props.activeTheme] === undefined
    ) {
      this.fetchCSS()
    }
  }

  render(): JSX.Element {
    return (
      <>
        <VariableKnobs
          theme={this.state.themes[this.props.activeTheme] ?? {}}
          themeOverrides={
            this.state.themeOverrides[this.props.activeTheme] ?? {}
          }
          onContribute={this.openContributeModal}
          onChange={this.setThemeOverrides}
        />

        {this.state.contributeModalIsOpen && (
          <ContributeForm
            baseTheme={this.state.themes[this.props.activeTheme] ?? {}}
            baseThemeName={this.props.activeTheme}
            themeOverrides={
              this.state.themeOverrides[this.props.activeTheme] ?? {}
            }
            onClose={this.closeContributeModal}
          />
        )}
      </>
    )
  }
}
