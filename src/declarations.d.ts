declare module 'mathjax-node' {
  function typeset(
    options: Record<string, boolean>,
    callback: (result: { css?: string; errors?: string[] }) => unknown
  ): unknown
}
