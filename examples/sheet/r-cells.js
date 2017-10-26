import template from './template'

export default function () {
  return template({
    'A': {
      width: 400
    }
  },{
    A1: 'Mean:', 'B1': 0,
    A2: 'Standard dev.', 'B2': 1,
    A3: 'Sample size', 'B3': 10000,
    A4: 'Colour', 'B4': 'grey',
    A5: {
      language: 'r',
      content: '= hist(rnorm(min(B3, 1e6), B1, B2), col=B4, breaks=50, xlim=c(-4,4), main="", xlab="")'
    }
  })
}
