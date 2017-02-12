const GeneralError = require('../utilities/general-error')

/**
 * Base class for all component converter errors
 */
class ComponentConverterError extends GeneralError {

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
class ComponentConverterUnknown extends ComponentConverterError {

  constructor (component, format) {
    super('Unknown format', component, format)
  }

}

module.exports = {
  ComponentConverterUnknown: ComponentConverterUnknown
}
