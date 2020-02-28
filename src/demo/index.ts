/**
 * Thema demo
 *
 * Provides an interface for switching of both example and theme.
 * Used for human 🤗 user acceptance testing 👍 and robot 🤖
 * visual regression testing.
 *
 * For HTML content and Javascript modules,
 * switching is achieved via Parcel's [dynamic importing of modules]
 * (https://parceljs.org/code_splitting.html). Note that this seems to load
 * content for all examples and all themes i.e. it is not truly lazy.
 *
 * For CSS, this `import()` approach did not work, maybe because it loads all the
 * CSS stylesheets into the global DOM. So, we take the approach of
 * enabling/disabling `<link>` elements.
 */

import { examples, resolveExample } from '../examples'
import { themes } from '../themes'
import { create, append, ready } from '../util'
import { styleEntry } from '../browser'
import { getCssVariables } from './parseCss'

const url = new URL(window.location.href)
let preview: HTMLIFrameElement | null | undefined
const getPreviewDoc = (): Document | null =>
  preview != null ? preview.contentDocument : null

const themeName = document.getElementById('themeName')
const themeVariables = document.getElementById('themeVariables')

/**
 * The keys used to refer to which example and
 * which theme the user wants to see.
 */
enum keys {
  EXAMPLE = 'example',
  THEME = 'theme',
  HEADER = 'header'
}

/**
 * Default values for options.
 */
const defaults: {
  EXAMPLE: string
  THEME: string
  HEADER: 'true' | 'false'
} = {
  EXAMPLE: examples.articleReadme,
  THEME: 'stencila',
  HEADER: 'true'
}

const getThemeCSS = (theme: string): string => {
  const req = new XMLHttpRequest()
  req.open('GET', `themes/${theme}/styles.css`, false)
  req.send(null)
  return req.responseText
}

// Theme customizer UI
const parseTheme = (theme: string): void => {
  // Find CSS variables from a stylesheet
  const css = getThemeCSS(theme)
  const vars = getCssVariables(css)

  // Build up a form label/input pairs for each variable
  const formEls = Object.entries(vars).reduce(
    (els: Element[], [name, value]) => {
      const label = create('label')
      label.textContent = name

      const input = create('input') as HTMLInputElement
      input.value = value

      return [...els, label, input]
    },
    []
  )

  if (themeName !== null) {
    themeName.textContent = theme
  }

  // Insert input fields into sidebar
  if (themeVariables !== null) {
    themeVariables.innerHTML = ''
    formEls.map(el => append(themeVariables, el))
  }
}

// Set an example
const exampleSet = (example: string): void => {
  // Update all the places that theme is set
  url.searchParams.set(keys.EXAMPLE, example)
  history.pushState(null, 'none', url.toString())
  sessionStorage.setItem(keys.EXAMPLE, example)
  if (exampleSelect !== null) exampleSelect.value = example

  // Change the preview document contents to the chosen example
  if (preview != null) {
    preview.setAttribute('src', `examples/${resolveExample(example)}.html`)
  }
}

// Initialize the example selector
const exampleSelect = document.querySelector<HTMLInputElement>(
  '#example-select'
)
if (exampleSelect !== null) {
  exampleSelect.innerHTML = Object.keys(examples)
    .map(
      ex =>
        `<option value="${ex}">${ex.replace(
          /^([a-z]+)([A-Z][a-z])*/g,
          '$1: $2'
        )}</option>`
    )
    .join('')
  exampleSelect.addEventListener('change', event => {
    const target = event.currentTarget as HTMLSelectElement
    if (target !== null) exampleSet(target.value)
  })
}

// Set the initial example
exampleSet(
  url.searchParams.get(keys.EXAMPLE) ??
    sessionStorage.getItem(keys.EXAMPLE) ??
    defaults.EXAMPLE
)

// Set a theme
const themeSet = (theme: string): void => {
  // Update all the places that theme is set
  url.searchParams.set(keys.THEME, theme)
  history.pushState(null, 'none', url.toString())
  sessionStorage.setItem(keys.THEME, theme)
  if (themeSelect !== null) themeSelect.value = theme

  parseTheme(theme)

  const themeStyles = create('link')
  themeStyles.setAttribute('rel', 'stylesheet')
  themeStyles.setAttribute('href', `/themes/${theme}/${styleEntry}`)
  themeStyles.setAttribute('id', 'themaStyles')

  const previewDoc = getPreviewDoc()
  const previewHead = previewDoc?.getElementsByTagName('head')[0]

  if (preview != null && previewHead != null) {
    const injectedStyle = previewDoc?.getElementById('themaStyles')

    if (injectedStyle != null) {
      injectedStyle.remove()
    }

    append(previewHead, themeStyles)
  }

  // Remove all appended theme scripts, and re-append chosen theme’s script
  // This causes the browser to re-evaluate the script
  // TODO: Check if this is still needed now that content is in an iframe
  if (previewDoc != null) {
    previewDoc.querySelectorAll('script.themeScript').forEach(node => {
      if (previewDoc != null) {
        previewDoc.body.removeChild(node)
      }
    })

    const themeScript = previewDoc.createElement('script')
    themeScript.type = 'text/javascript'
    themeScript.src = `themes/${theme}/index.js`
    themeScript.classList.add('themeScript')

    previewDoc.body.appendChild(themeScript)

    // Add delay before dispatching ready event, giving newly added theme script time to be parsed
    // This causes the dom/ready() helper function to perform any scheduled functions.
    setTimeout(() => {
      if (previewDoc != null) {
        previewDoc.dispatchEvent(
          new Event('DOMContentLoaded', {
            bubbles: true,
            cancelable: true
          })
        )
      }
    }, 300)
  }
}

// Set a theme when it is selected from the theme selector
const themeSelect = document.querySelector<HTMLInputElement>('#theme-select')
if (themeSelect !== null) {
  themeSelect.innerHTML = Object.keys(themes)
    .map(
      theme =>
        `<option value="${theme}">${theme.replace(
          /^([a-z])([A-Z][a-z])*/g,
          '$1$2'
        )}</option>`
    )
    .join('')

  themeSelect.addEventListener('change', event => {
    const target = event.currentTarget as HTMLSelectElement
    if (target !== null) themeSet(target.value)
  })
}

// Change preview iframe's size to simulate a mobile view
const mobileView = (e: MouseEvent): void => {
  e.preventDefault()
  if (preview != null) {
    preview.classList.add('mobile')
  }
}

// Make preview iframe full width
const desktopView = (e: MouseEvent): void => {
  e.preventDefault()
  if (preview != null) {
    preview.classList.remove('mobile')
  }
}

ready(() => {
  // TOOD: Check to see if this is triggered each time theme/example is changed
  preview = document.getElementsByTagName('iframe')[0] ?? null

  // Set display of header
  const header = document.querySelector<HTMLInputElement>('#header')
  if (header !== null) {
    header.style.display =
      url.searchParams.get(keys.HEADER) === 'false' ? 'none' : 'block'
  }

  // Attach event handler to theme editor sidebar
  const mobileButton = document.getElementById('mobileView')
  if (mobileButton !== null) {
    mobileButton.addEventListener('mouseup', mobileView)
  }

  const desktopButton = document.getElementById('desktopView')
  if (desktopButton !== null) {
    desktopButton.addEventListener('mouseup', desktopView)
  }

  preview.addEventListener('load', function() {
    // Set the initial theme
    themeSet(
      url.searchParams.get(keys.THEME) ??
        sessionStorage.getItem(keys.THEME) ??
        defaults.THEME
    )
  })
})
