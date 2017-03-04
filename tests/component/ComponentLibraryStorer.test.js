import test from 'tape'

import ComponentLibraryStorer from '../../src/component/ComponentLibraryStorer'

test('ComponentLibraryStorer.split', t => {
  let s = new ComponentLibraryStorer()

  t.deepEqual(s.split('lib://README.md'), { file: 'README.md', ref: 'master', repo: 'lib', user: 'stencila' })
  t.deepEqual(s.split('lib://README.md@0.1.0'), { file: 'README.md', ref: '0.1.0', repo: 'lib', user: 'stencila' })

  t.end()
})
