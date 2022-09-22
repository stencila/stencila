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
  Develop = 5,
  Edit = 6,
  Write = 7,
  Shell = 8,
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
    case Mode.Develop:
      return 'Create, update and delete dynamic elements'
    case Mode.Edit:
      return 'Create, update and delete content elements'
    case Mode.Write:
      return 'Create, update and delete all elements'
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
    case Mode.Develop:
      return 'Develop'
    case Mode.Edit:
      return 'Edit'
    case Mode.Write:
      return 'Write'
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
      return 'wifi'
    case Mode.Inspect:
      return 'search'
    case Mode.Interact:
      return 'sliders'
    case Mode.Alter:
      return 'wrench-adjustable'
    case Mode.Develop:
      return 'code'
    case Mode.Edit:
      return 'pencil'
    case Mode.Write:
      return 'braces-asterisk'
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
      return DevStatus.Stable
    case Mode.Dynamic:
    case Mode.Inspect:
      return DevStatus.Beta
    case Mode.Interact:
    case Mode.Alter:
      return DevStatus.Alpha
    case Mode.Develop:
      return DevStatus.ComingSoon
    case Mode.Edit:
    case Mode.Write:
      return DevStatus.Planned
    case Mode.Shell:
      return DevStatus.Beta
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
    case 'develop':
      return Mode.Develop
    case 'edit':
      return Mode.Edit
    case 'write':
      return Mode.Write
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
  return modeFromString(window.stencilaConfig.mode) ?? Mode.Static
}
