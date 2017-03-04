import GeneralError from '../utilities/general-error'

/**
 * Base class for all component storer errors
 */
export class ComponentStorerError extends GeneralError {

  constructor (message, storer, address) {
    super(message, {
      storer: storer,
      address: address
    })
  }
}

/**
 * Error thrown when there is no known storere for a scheme
 */
export class ComponentStorerUnknown extends ComponentStorerError {

  constructor (scheme, address) {
    super('Unknown scheme', null, address)
  }

}

/**
 * Error thrown when an address is malfomed
 */
export class ComponentStorerMalformed extends ComponentStorerError {

  constructor (storer, address) {
    super('Address supplied is malfomed', storer, address)
  }

}

/**
 * Error thrown when an attempt is made to read or write
 * to an address that can not be found. e.g. HTTP 404
 * Usually, user needs to be asked to correct the address
 */
export class ComponentStorerUnfound extends ComponentStorerError {

  constructor (storer, address) {
    super('Address not found by component storer', storer, address)
  }

}

/**
 * Error thrown when an attempt is made to write using a component
 * storer than is unable to write. Usually, this will mean that the
 * component should be 'saved as' to a different address.
 */
export class ComponentStorerUnwritable extends ComponentStorerError {

  constructor (storer, address) {
    super('This component storer can not write', storer, address)
  }

}
