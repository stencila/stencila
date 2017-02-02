/**
 * Create a new `Error` with message and details
 *
 * @param  {String} message Message for the error
 * @param  {Object} details Details associated with the error
 * @return {Error}          New error
 */
function error (message, details) {
  return new Error(message + ':' + JSON.stringify(details))
}

module.exports = error
