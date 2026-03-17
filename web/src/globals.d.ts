declare module '*.svg?raw' {
  const content: string
  export default content
}

declare module 'vega-embed' {
  function vegaEmbed(
    el: HTMLElement | string,
    spec: object,
    opts?: Record<string, unknown>
  ): Promise<{ finalize: () => void }>
  export default vegaEmbed
}

declare module 'idiomorph/dist/idiomorph.esm.js' {
  interface Idiomorph {
    morph(element: Element, other: Element | string): void
  }

  const Idiomorph: Idiomorph
}
