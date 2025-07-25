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

declare interface Window {
  // Global to turn off interactive elements on special images
  // e.g. Plotly, Leaflet
  STENCILA_STATIC_MODE?: boolean
}
