export const format = (links: Element[]): void => {
  links.forEach((link: Element): void => {
    if (link.hasAttribute('target') === false) {
      link.setAttribute('target', '_parent')
    }
  })
}
