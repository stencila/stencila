import { createContext, createSignal, For, Show, Suspense } from 'solid-js'
import { Link } from 'solid-meta'

/**
 * The theme context that is implicitly passed down
 * the component tree.
 *
 * Can be used by components to alter rendering based on
 * properties of the current theme.
 */

interface ThemeContext {
  url: string
}

export const ThemeContext = createContext<ThemeContext>()

function themeFallback(theme?: string): string {
  if (theme === undefined) {
    const params = new URLSearchParams(window.location.search)
    theme = params.get('theme') ?? 'wilmore'
  }
  return theme
}

function themeUrl(theme: string): string {
  return theme.startsWith('http')
    ? theme
    : `https://unpkg.com/@stencila/thema/dist/themes/${theme}/styles.css`
}

/**
 * A `ThemeContext` provider intended for rendering a document
 * to a string (i.e. server-side rendering).
 *
 * Uses `solid-meta` to add the theme's assets to the `<head>`.
 *
 * @param props.document The `CreativeWork` to be rendered.
 */
export function ThemeRenderer(props: { theme?: string; children: any }) {
  const url = () => themeUrl(themeFallback(props.theme))
  return (
    <ThemeContext.Provider value={{ url: url() }}>
      <Link href={url()} type="text/css" rel="stylesheet" />
      <div data-itemscope="root">{props.children}</div>
    </ThemeContext.Provider>
  )
}

/**
 * A `ThemeContext` provider that inserts a `<link>` in to the page and
 * does not show children until it is loaded.
 *
 * Avoids a "flash-of-un-styled-content" by only showing content when the
 * style has loaded. Removes any previously added theme stylesheets.
 *
 * @param props.theme The name of a Stencila theme or a URL of a `styles.css` file
 */
export function ThemeLinker(props: { theme?: string; children: any }) {
  const [ready, setReady] = createSignal(false)

  const url = (): string => {
    setReady(false)

    // Remove any existing theme stylesheet
    document
      .querySelectorAll('link#theme')
      .forEach((link) => link?.parentNode?.removeChild(link))

    // Add a new theme stylesheet
    const url = themeUrl(themeFallback(props.theme))
    const link = document.createElement('link')
    link.id = 'theme'
    link.type = 'text/css'
    link.rel = 'stylesheet'
    link.href = url
    link.onload = () => setReady(true)
    document.head.appendChild(link)

    return url
  }

  return (
    <ThemeContext.Provider value={{ url: url() }}>
      <Show when={ready()}>
        <div data-itemscope="root">{props.children}</div>
      </Show>
    </ThemeContext.Provider>
  )
}

/**
 * A `ThemeContext` provider that allows switching of the theme.
 *
 * @param props.themes The names of a Stencila themes to allow users to select from
 */
export function ThemeSwitcher(props: {
  theme?: string
  themes?: string
  children: any
}) {
  const [theme, setTheme] = createSignal(themeFallback(props.theme))

  let themes = props.themes ?? [
    'elife',
    'f1000',
    'latex',
    'nature',
    'plos',
    'giga',
    'stencila',
    'tufte',
    'wilmore',
  ]

  return (
    <>
      <select onChange={(event) => setTheme(event.target.value)}>
        <For each={themes}>
          {(option) => (
            <option value={option} selected={option === theme()}>
              {option}
            </option>
          )}
        </For>
      </select>
      <ThemeLinker theme={theme()}>{props.children}</ThemeLinker>
    </>
  )
}
