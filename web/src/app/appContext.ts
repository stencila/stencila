import { createContext } from '@lit/context';

type View = 
  | "dynamic"
  | "live"
  | "source"
  | "static"
  | "visual"

type AppContext = {
  view: View
}

export const appContext = createContext<AppContext>(Symbol('app-context'))
export type { AppContext }