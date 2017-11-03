import template from './template'

export default function () {
  return template({
    // As part of excercises these get changed to...
    A: {
      //name: 'temperate',
      //type: 'number',
      width: 300,
    }, 
    B: {
      //name: 'sales',
      //type: 'number'
      width: 300,
    }
  },{
    A1: 18, A2: 20, A3: 24, A4: 26, A5: 25, A6: 20, A7: 17, A8: 18, A9: 28,
    B1: 50, B2: 126, B3: 118, B4: 126, B5: 280, B6: 102, B7: 93, B8: 32, B9: 103,
    A11: {
      language: 'r',
      content: '= summary(A1:B9)'
    },
    B11: {
      language: 'r',
      content: '= capture.output(summary(lm(B~A, A1:B9)))'
    },
    A12: {
      language: 'r',
      content: '= plot(A1:B9, cex=1.3, pch=16)'
    },
  })
}
