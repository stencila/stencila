import { Themes } from '..'

interface PrivateThemes {
  skeleton: 'skeleton'
}

export const modules: {
  [key in keyof Themes & PrivateThemes]: Promise<any>
} = {
  elife: import('./elife'),
  nature: import('./nature'),
  plos: import('./plos'),
  stencila: import('./stencila'),
  skeleton: import('./skeleton')
}
