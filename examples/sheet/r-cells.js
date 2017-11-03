import template from './template'

export default function () {
  return template({
  },{
    A1: 'Intercept', B1: 0,
    A2: 'Slope',     B2: 1,
    A3: 'Epsilon',   B3: 3,
    A5: 'X',         B5: 10,
    A6: 'Y (predicted)', B6: '= B1 + B2 * B5',
    A7: {
      language: 'r',
      content: '= plot(B1 + seq(0,B5,0.1) * B2 + rnorm(B5/0.1, 0, B3), pch=16, cex=2, col=hsv(0.6,0.9,0.9), ylab="Y", xlab="X")'
    }
  })
}
