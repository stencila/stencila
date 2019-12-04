import { Themes } from '..'

export const modules: { [key in keyof Themes]: Promise<any> } = {
  elife: import('./eLife'),
  nature: import('./nature'),
  plos: import('./plos'),
  stencila: import('./stencila')
}
