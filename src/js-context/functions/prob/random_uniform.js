import random_generate_ from './random_generate_'

function random_uniform(n = 1, min = 0, max = 1) {
  let diff = max - min
  return random_generate_(n, () => min + Math.random() * diff)
}
random_uniform.pars = ['n', 'min', 'max']

export default random_uniform
