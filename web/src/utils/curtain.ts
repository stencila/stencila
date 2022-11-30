/**
 * Wait for certain Stencila custom elements to be ready
 */
export async function waitForElems(elements: string[]) {
  document.body.classList.remove('ready')
  await Promise.allSettled(
    elements.map((element) => customElements.whenDefined(`stencila-${element}`))
  )
  document.body.classList.add('ready')
}
