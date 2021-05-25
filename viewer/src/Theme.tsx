import { createContext } from 'solid-js'
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

function themeUrl(theme?: string): string {
  if (theme === undefined) {
    const params = new URLSearchParams(window.location.search)
    theme = params.get('theme') ?? 'wilmore'
  }

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
  return (
    <ThemeContext.Provider value={{ url: themeUrl(props.theme) }}>
      <Link href={themeUrl(props.theme)} type="text/css" rel="stylesheet" />
      <div data-itemscope="root">{props.children}</div>
    </ThemeContext.Provider>
  )
}

/**
 * A `ThemeContext` provider that inserts a `<link>` in to the page and
 * does not show children until it is loaded.
 *
 * @param props.theme The name of a Stencila theme or a URL of a `styles.css` file
 */
export function ThemeLinker(props: { theme?: string; children: any }) {
  return (
    <ThemeContext.Provider value={{ url: themeUrl(props.theme) }}>
      <link href={themeUrl(props.theme)} type="text/css" rel="stylesheet" />
      <div data-itemscope="root">{props.children}</div>
    </ThemeContext.Provider>
  )
}
