import random_uniform from './random_uniform'

/**
 * Generate random numbers from a probability distribution
 *
 * This function delegates to random functions for specific distributions
 * e.g. `random_normal`, `random_poisson`. The default distribution is the uniform
 * with parameters min = 0 and max = 1.
 *
 * @example
 * 
 * random()
 * 0.3784768
 *
 * random(4)
 * [0.9282889, 0.1122377, 0.2396080, 0.9305602]
 *
 * //
 * 
 * random(4, 'normal', 11.3, 2.4)
 *
 * // Same as...
 * 
 * random_normal(4, 11.3, 2.4)
 * 
 * @param  {number}    n    Sample size. Defaults to 1
 * @param  {string}    dist Name of distribution
 * @param  {...[type]} args Parameters of the probability distribution (e.e. mean and standard deviation)
 * @return {[type]}         [description]
 */
function random (n, dist, ...args) {
  n = n || 1
  dist = dist || 'uniform'

  if (dist === 'uniform') return random_uniform(n, ...args)
  else throw(new Error(`Unknown distribution: ${dist}`))
}
random.args = ['n', 'dist']
export default random
