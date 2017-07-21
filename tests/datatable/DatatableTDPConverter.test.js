import test from 'tape'

import DatatableTDPConverter from '../../src/datatable/DatatableTDPConverter'
import MemoryStorer from '../../src/host/MemoryStorer'

test('DatatableTDPConverter:match', t => {
  let c = new DatatableTDPConverter()
  
  let storer1 = new MemoryStorer({
    'datapackage.json': '{}',
    'data.csv': 'col1,col2\n1,2\n3,4\n'
  })

  let storer2 = new MemoryStorer({
    'data.csv': 'col1,col2\n1,2\n3,4\n'
  })

  t.plan(4)
  
  c.match('datapackage.json', storer1).then(result => {
    t.ok(result)
  })

  c.match('not-a-data-package.json', storer1).then(result => {
    t.notOk(result)
  })

  c.match('data.csv', storer1).then(result => {
    t.ok(result)
  }, 'has a sibling datapackage.json')

  c.match('data.csv', storer2).then(result => {
    t.notOk(result)
  }, 'no sibling datapackage.json')
})

test.skip('DatatableTDPConverter:import', t => {
  let c = new DatatableTDPConverter()
  t.throws(c.import, 'DatatableTDPConverter.import() must be implemented in derived class')
  t.end()
})

test.skip('DatatableTDPConverter:export', t => {
  let c = new DatatableTDPConverter()
  t.throws(c.export, 'DatatableTDPConverter.export() must be implemented in derived class')
  t.end()
})
