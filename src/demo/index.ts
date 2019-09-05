/**
 * Thema demo app
 *
 * Implements switching of content and theme.
 *
 * For HTML content and Javascript modules,
 * switching is achieved via Parcel's [dynamic importing of modules]
 * (https://parceljs.org/code_splitting.html). Note that this seems to load
 * content for all examples and all themes i.e. it is not truly lazy.
 *
 * For CSS, this `import()` approach did not work, maybe because it loads all the
 * CSS stylesheets into the global DOM. So, we take the approach of enablig/diabling `<link>`
 * elements.
 */

import { examples } from '../examples'
import { modules } from '..'

const contentSelect = document.querySelector('#content-select')
if (contentSelect) {
  contentSelect.addEventListener('change', async event =>
    await contentSet((event.target as HTMLInputElement).value)
  )
}

const contentSet = async (example: string) => {
  // Load the HTML content
  const content = await examples[example]
  const html = content.default

  // Set the content of `<main>` to the content of the example `<body>`
  const dom = new DOMParser().parseFromString(html, 'text/html')
  const main = document.getElementsByTagName('main')[0]
  main.innerHTML = dom.body.innerHTML
}

// Initial content...
contentSet('article-drosophila')

const themeSelect = document.querySelector('#theme-select')
if (themeSelect) {
  themeSelect.addEventListener('change', async event => {
    const element = event.target as HTMLInputElement
    const theme = element.value

    // Load the theme's Javascript module and run it's `init()` function
    // @ts-ignore
    const mod = await modules[theme]
    if (mod !== undefined && 'init' in mod) mod.init()

    // Enable the theme's stylesheet and disable all others
    const main = document.getElementsByTagName('main')[0]
    document
      .querySelectorAll('link.theme[rel="stylesheet"]')
      .forEach(node => (node as HTMLInputElement).disabled = node.id !== theme)
  })
}
