interface FilterHighlightAllPlugin {
  reject: {
    add: (
      predicate: (el: { element: Element; language: string }) => boolean
    ) => void
  }
}

export interface PrismJsPlugins {
  filterHighlightAll: FilterHighlightAllPlugin
}
