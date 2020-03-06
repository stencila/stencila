import { diff, objToVars, ThemeObject } from '.'
import { styleEntry, themes } from '../../browser'
import { append, create, prepend } from '../../util'
import { keys } from './index'
import {
  forceReady,
  getExample,
  getPreviewDoc,
  getPreviewHead
} from './preview'

export const getThemeCSS = (theme: string): string => {
  const req = new XMLHttpRequest()
  req.open('GET', `themes/${theme}/styles.css`, false)
  req.send(null)
  return req.responseText
}

export const getTheme = (): string => {
  return (
    new URL(window.location.href).searchParams.get(keys.THEME) ??
    sessionStorage.getItem(keys.THEME) ??
    themes.stencila
  )
}

export const themeSet = (theme: string): void => {
  // Update all the places that theme is set
  const url = new URL(window.location.href)

  url.searchParams.set(keys.EXAMPLE, getExample())
  url.searchParams.set(keys.THEME, theme)
  history.replaceState(null, 'none', url.toString())
  sessionStorage.setItem(keys.THEME, theme)

  const themeStyles = create('link')
  themeStyles.setAttribute('rel', 'stylesheet')
  themeStyles.setAttribute('href', `/themes/${theme}/${styleEntry}`)
  themeStyles.setAttribute('id', 'themaStyles')

  const previewDoc = getPreviewDoc()
  const previewHead = getPreviewHead()

  if (previewHead !== null) {
    const injectedStyle = previewDoc?.getElementById('themaStyles')

    if (injectedStyle !== undefined && injectedStyle !== null) {
      injectedStyle.remove()
    }

    prepend(previewHead, themeStyles)
  }

  // Remove all appended theme scripts, and re-append chosen theme’s script
  // This causes the browser to re-evaluate the script
  if (previewDoc !== null) {
    previewDoc.querySelectorAll('script.themeScript').forEach(node => {
      if (previewDoc != null) {
        previewDoc.body.removeChild(node)
      }
    })

    const themeScript = previewDoc.createElement('script')
    themeScript.type = 'text/javascript'
    themeScript.src = `/themes/${theme}/index.js`
    themeScript.classList.add('themeScript')

    previewDoc.body.appendChild(themeScript)

    // Add delay before dispatching ready event, giving newly added theme script time to be parsed
    // This causes the dom/ready() helper function to perform any scheduled functions.
    setTimeout(() => {
      forceReady(previewDoc)
    }, 300)
  }
}

export const upsertThemeOverrides = (
  baseTheme: ThemeObject,
  newTheme: ThemeObject
): void => {
  // Get values from sidebar, find changed values, and convert to CSS rule definition
  const cssVars = objToVars(diff(baseTheme, newTheme))

  // Inject updated rules into preview `head` element
  const head = getPreviewHead()

  if (head != null) {
    // Remove old updated styles if they exist
    const oldStyles = head.querySelector('.varOverrides')
    if (oldStyles !== null) {
      oldStyles.remove()
    }

    const style = create('style', cssVars)
    style.classList.add('varOverrides')

    append(head, style)
  }
}
