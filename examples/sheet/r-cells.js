import template from './template'

export default function () {
  return template({
    'B': {
      width: 200
    }
  },{
    'A1': '1',
    'A2': '2',
    'B3': {
      language: 'r',
      content: '= plot(A1,A2)'
    }
  })
}
