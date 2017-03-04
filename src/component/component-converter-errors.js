import GeneralError from '../utilities/general-error'

/**
 * Base class for all component converter errors
 */
export class ComponentConverterError extends GeneralError {

  constructor (message, component, format) {
    super(message, {
      component: component,
      format: format
    })
  }

}

/**
 * Error thrown when there is no known converter for a format
 */
export class ComponentConverterUnknown extends ComponentConverterError {

  constructor (component, format) {
    super('Unknown format', component, format)
  }

}
