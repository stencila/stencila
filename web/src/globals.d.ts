declare module '*.svg' {
  const content: string
  export default content
}

declare module 'idiomorph/dist/idiomorph.esm.js' {
  interface Idiomorph {
    morph(element: Element, other: Element | string): void
  }

  const Idiomorph: Idiomorph
}
