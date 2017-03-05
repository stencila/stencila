import GeneralError from '../utilities/general-error'

/**
 * Base class for all component converter errors
 */
export class ComponentConverterError extends GeneralError {

  constructor (message, type, format) {
    super(message, {
      type: type,
      format: format
    })
  }

}

/**
 * Error thrown when there is no known converter for a format
 */
export class ComponentConverterUnknown extends ComponentConverterError {

  constructor (type, format) {
    super('Unknown format', type, format)
  }

}
