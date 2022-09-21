/**
 * The user mode for the document
 */
export const enum Mode {
  Static = 0,
  Read = 1,
  Interact = 2,
  Inspect = 3,
  Alter = 4,
  Develop = 5,
  Edit = 6,
  Write = 7,
}

/**
 * Get the description of a mode
 */
export function modeDesc(mode: Mode): string {
  switch (mode) {
    case Mode.Static:
      return 'Read a static view'
    case Mode.Read:
      return 'Read a dynamic view'
    case Mode.Interact:
      return 'Interact with dynamic elements'
    case Mode.Inspect:
      return 'Inspect and interact with dynamic elements'
    case Mode.Alter:
      return 'Alter and interact with dynamic elements'
    case Mode.Develop:
      return 'Create, update and delete dynamic elements'
    case Mode.Edit:
      return 'Create, update and delete content elements'
    case Mode.Write:
      return 'Create, update and delete all elements'
  }
}

/**
 * Convert a `Mode` to a string
 */
export function modeToString(mode: Mode): string {
  switch (mode) {
    case Mode.Static:
      return 'Static'
    case Mode.Read:
      return 'Read'
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
  }
}

/**
 * Convert a string into a `Mode`
 */
export function modeFromString(mode: string): Mode {
  switch (mode.toLowerCase()) {
    case 'static':
      return Mode.Static
    case 'read':
      return Mode.Read
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
    default:
      throw new Error(`Could not convert string '${mode}' to a mode`)
  }
}
