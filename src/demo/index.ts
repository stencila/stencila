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

const url = new URL(window.location.href)

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

// The `init` function of the current theme
let themeInit: () => void

// Set an example
const exampleSet = (example: string): void => {
  // Update all the places that theme is set
  url.searchParams.set(keys.EXAMPLE, example)
  history.pushState(null, 'none', url.toString())
  sessionStorage.setItem(keys.EXAMPLE, example)
  if (exampleSelect !== null) exampleSelect.value = example

  // Load the HTML content
  const req = new XMLHttpRequest()
  req.open('GET', `examples/${resolveExample(example)}.html`, false)
  req.send(null)
  const html = req.responseText

  // Set the content of `<main>` to the content of the example `<body>`
  const dom = new DOMParser().parseFromString(html, 'text/html')
  const main = document.getElementsByTagName('main')[0]
  main.innerHTML = dom.body.innerHTML

  // Run the `init` function of the current theme
  if (themeInit !== undefined) themeInit()
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

  // Enable the theme's CSS
  document
    .querySelectorAll('link.theme[rel="stylesheet"]')
    .forEach(node => ((node as HTMLInputElement).disabled = node.id !== theme))

  // Append remove all theme scripts, and re-append chosen theme’s script
  // This causes the browser to re-evaluate the script
  document.querySelectorAll('script.themeScript').forEach(node => {
    document.body.removeChild(node)
  })

  const themeScript = document.createElement('script')
  themeScript.type = 'text/javascript'
  themeScript.src = `themes/${theme}/index.js`
  themeScript.classList.add('themeScript')

  document.body.appendChild(themeScript)

  // Add delay before dispatching ready event, giving newly added theme script time to be parsed
  // This causes the dom/ready() helper function to perform any scheduled functions.
  setTimeout(() => {
    document.dispatchEvent(
      new Event('DOMContentLoaded', {
        bubbles: true,
        cancelable: true
      })
    )
  }, 300)
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

// Set the initial theme
themeSet(
  url.searchParams.get(keys.THEME) ??
    sessionStorage.getItem(keys.THEME) ??
    defaults.THEME
)

// Set display of header
const header = document.querySelector<HTMLInputElement>('#header')
if (header !== null) {
  header.style.display =
    url.searchParams.get(keys.HEADER) === 'false' ? 'none' : 'block'
}
