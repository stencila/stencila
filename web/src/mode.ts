import { IconName } from './components/base/icon'
import { DevStatus } from './dev-status'

/**
 * The user mode for the document
 */
export const enum Mode {
  Static = 0,
  Dynamic = 1,
  Interact = 2,
  Inspect = 3,
  Alter = 4,
  Design = 5,
  Edit = 6,
  Develop = 7,
  Write = 8,
  Code = 9,
  Shell = 10,
}

/**
 * Get the description of a mode
 */
export function modeDesc(mode: Mode): string {
  switch (mode) {
    case Mode.Static:
      return 'Static version with no realtime updates'
    case Mode.Dynamic:
      return 'Realtime updates to dynamic elements'
    case Mode.Interact:
      return 'Interact with dynamic elements'
    case Mode.Inspect:
      return 'Inspect and interact with dynamic elements'
    case Mode.Alter:
      return 'Alter and re-run dynamic elements'
    case Mode.Design:
      return 'Create, update and delete style elements'
    case Mode.Edit:
      return 'Create, update and delete content elements'
    case Mode.Develop:
      return 'Create, update and delete style and dynamic elements'
    case Mode.Write:
      return 'Create, update and delete all elements'
    case Mode.Code:
      return 'Use a code editor to modify the document'
    case Mode.Shell:
      return 'Use a shell terminal to do anything!'
  }
}

/**
 * Get the label for a mode
 */
export function modeLabel(mode: Mode): string {
  switch (mode) {
    case Mode.Static:
      return 'Static'
    case Mode.Dynamic:
      return 'Dynamic'
    case Mode.Interact:
      return 'Interact'
    case Mode.Inspect:
      return 'Inspect'
    case Mode.Alter:
      return 'Alter'
    case Mode.Design:
      return 'Design'
    case Mode.Edit:
      return 'Edit'
    case Mode.Develop:
      return 'Develop'
    case Mode.Write:
      return 'Write'
    case Mode.Code:
      return 'Code'
    case Mode.Shell:
      return 'Shell'
  }
}

/**
 * Get the icon for a mode
 */
export function modeIcon(mode: Mode): IconName {
  switch (mode) {
    case Mode.Static:
      return 'wifi-off'
    case Mode.Dynamic:
      return 'broadcast-pin'
    case Mode.Inspect:
      return 'eye'
    case Mode.Interact:
      return 'sliders'
    case Mode.Alter:
      return 'wrench-adjustable'
    case Mode.Design:
      return 'palette'
    case Mode.Edit:
      return 'pencil'
    case Mode.Develop:
      return 'code'
    case Mode.Write:
      return 'braces-asterisk'
    case Mode.Code:
      return 'code-square'
    case Mode.Shell:
      return 'terminal-fill'
  }
}

/**
 * Get the development status of a mode
 */
export function modeDevStatus(mode: Mode): DevStatus {
  switch (mode) {
    case Mode.Static:
      return DevStatus.Beta
    case Mode.Dynamic:
    case Mode.Inspect:
    case Mode.Interact:
    case Mode.Alter:
    case Mode.Design:
    case Mode.Edit:
    case Mode.Develop:
    case Mode.Write:
    case Mode.Code:
    case Mode.Shell:
      return DevStatus.Alpha
  }
}

/**
 * Convert a string into a `Mode`
 */
export function modeFromString(mode: string): Mode {
  switch (mode.toLowerCase()) {
    case 'static':
      return Mode.Static
    case 'dynamic':
      return Mode.Dynamic
    case 'interact':
      return Mode.Interact
    case 'inspect':
      return Mode.Inspect
    case 'alter':
      return Mode.Alter
    case 'design':
      return Mode.Design
    case 'edit':
      return Mode.Edit
    case 'develop':
      return Mode.Develop
    case 'write':
      return Mode.Write
    case 'code':
      return Mode.Code
    case 'shell':
      return Mode.Shell
    default:
      throw new Error(`Could not convert string '${mode}' to a mode`)
  }
}

/**
 * Get the current mode from the config
 */
export function currentMode(): Mode {
  return modeFromString(window.stencilaConfig.mode ?? 'static')
}

/**
 * Does the current mode allow for reading code elements?
 */
export function isCodeReadable(): boolean {
  const mode = currentMode()
  return mode >= Mode.Inspect
}

/**
 * Does the current mode allow for create/update/delete of code elements?
 */
export function isCodeWriteable(): boolean {
  const mode = currentMode()
  return mode == Mode.Alter || mode == Mode.Develop || mode == Mode.Write
}

/**
 * Does the current mode allow for executing code elements?
 */
export function isCodeExecutable(): boolean {
  const mode = currentMode()
  return mode >= Mode.Interact && mode !== Mode.Design && mode !== Mode.Edit
}

/**
 * Does the current mode allow for reading style elements?
 */
export function isStyleReadable(): boolean {
  const mode = currentMode()
  return mode >= Mode.Inspect
}

/**
 * Does the current mode allow for create/update/delete of style elements?
 */
export function isStyleWriteable(): boolean {
  const mode = currentMode()
  return mode === Mode.Design || mode === Mode.Develop || mode === Mode.Write
}

/**
 * Does the current mode allow for reading content elements?
 *
 * All modes allow this and this function is provided mainly for completeness
 * and in case at some point it needs to depend on mode.
 */
export function isContentReadable(): boolean {
  return true
}

/**
 * Does the current mode allow for create/update/delete of content elements?
 */
export function isContentWriteable(): boolean {
  const mode = currentMode()
  return mode === Mode.Edit || mode === Mode.Write
}
