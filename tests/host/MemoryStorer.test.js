import test from 'tape'

import MemoryStorer from '../../src/host/MemoryStorer'

let files = {
  'file-1.txt':'file-1-content',
  'file-2.txt':'file-2-content',
  'dir-1/file-3.txt':'file-3-content',
  'dir-2/file-4.txt':'file-4-content',
  'dir-2/file-5.txt':'file-5-content',
}

test('MemoryStorer:readFile', t => {
  let s = new MemoryStorer(Object.assign({},files))
  t.plan(3)

  s.readFile('file-1.txt').then(data => {
    t.equal(data, 'file-1-content')
  }).catch(err => {
    t.error(err)
  })

  s.readFile('dir-2/file-4.txt').then(data => {
    t.equal(data, 'file-4-content')
  }).catch(err => {
    t.error(err)
  })

  s.readFile('non-existent').catch(err => {
    t.equal(err.message, 'File not found')
  })
})

test('MemoryStorer:writeFile', t => {
  let s = new MemoryStorer(Object.assign({},files))
  t.plan(2)

  s.writeFile('dir-2/file-4.txt', 'file-4-new-content').then(() => {
    return s.readFile('dir-2/file-4.txt')
  }).then(data => {
    t.equal(data, 'file-4-new-content')
  })

  s.writeFile('dir-2/file-6.txt', 'file-6-content').then(() => {
    return s.readFile('dir-2/file-6.txt')
  }).then(data => {
    t.equal(data, 'file-6-content')
  })
})

test('MemoryStorer:readDir', t => {
  let s = new MemoryStorer(Object.assign({},files))
  t.plan(3)

  s.readDir('').then((files) => {
    t.deepEqual(files, ['file-1.txt', 'file-2.txt'])
  })

  s.readDir('dir-1').then((files) => {
    t.deepEqual(files, ['file-3.txt'])
  })

  s.readDir('dir-2').then((files) => {
    t.deepEqual(files, ['file-4.txt', 'file-5.txt'])
  })
})
