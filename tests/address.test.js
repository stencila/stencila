import test from 'tape'

import {long, short, split, scheme, storer, path, format, version} from '../src/address'

test('address.long', t => {
  t.equal(long('new://document'), 'new://document')
  t.equal(long('+document'), 'new://document')
  t.equal(long('*aaaaaaaa'), 'local://aaaaaaaa')

  t.equal(long('file:///report.md'), 'file:///report.md')
  t.equal(long('file:/report.md'), 'file:///report.md')
  t.equal(long('file:report.md'), 'file:///report.md')
  t.equal(long('./report.docx'), 'file://./report.docx')
  t.equal(long('/some/dir/report.docx'), 'file:///some/dir/report.docx')
  t.equal(long('~/report.docx'), 'file://~/report.docx')

  t.equal(long('http:foo.com/report.md'), 'http://foo.com/report.md')
  t.equal(long('https://foo.com/report.md'), 'https://foo.com/report.md')

  t.equal(long('gh:user/repo/report.md'), 'github://user/repo/report.md')
  t.equal(long('gh:/user/repo/report.md'), 'github://user/repo/report.md')
  t.equal(long('gh://user/repo/report.md'), 'github://user/repo/report.md')
  t.equal(long('github:user/repo/report.md'), 'github://user/repo/report.md')
  t.equal(long('github:/user/repo/report.md'), 'github://user/repo/report.md')
  t.equal(long('github://user/repo/report.md'), 'github://user/repo/report.md')

  t.equal(long('stats/t-test'), 'lib://stats/t-test')

  t.throws(() => long('foo:bar'), 'unknown scheme alias')

  t.end()
})

test('address.short', t => {
  t.equal(short('new://document'), '+document')
  t.equal(short('local://aaaaaaaa'), '*aaaaaaaa')
  t.equal(short('file://report.docx'), 'file:report.docx')
  t.equal(short('https://foo.com/report.md'), 'https:foo.com/report.md')
  t.equal(short('github://foo/bar/report.md'), 'gh:foo/bar/report.md')
  t.equal(short('lib://stats/t-test'), 'stats/t-test')
  t.end()
})

test('address.short+long', t => {
  let f = address => short(long(address))
  t.equal(f('+document'), '+document')
  t.equal(f('new://document'), '+document')
  t.equal(f('*aaaaaaaa'), '*aaaaaaaa')
  t.equal(f('local://aaaaaaaa'), '*aaaaaaaa')
  t.equal(f('gh:foo/bar/report.md'), 'gh:foo/bar/report.md')
  t.equal(f('gh:foo/bar/report.md@1.1.0'), 'gh:foo/bar/report.md@1.1.0')
  t.end()
})

test('address.split', t => {
  t.deepEqual(split('+document'), {
    scheme: 'new',
    path: 'document',
    format: null,
    version: null
  })

  t.deepEqual(split('*aaaaaaaa'), {
    scheme: 'local',
    path: 'aaaaaaaa',
    format: null,
    version: null
  })

  t.deepEqual(split('stats/t-test'), {
    scheme: 'lib',
    path: 'stats/t-test',
    format: null,
    version: null
  })

  t.deepEqual(split('stats/t-test@1.1.0'), {
    scheme: 'lib',
    path: 'stats/t-test',
    format: null,
    version: '1.1.0'
  })

  t.deepEqual(split('http://foo/bar'), {
    scheme: 'http',
    path: 'foo/bar',
    format: null,
    version: null
  })

  t.deepEqual(split('gh://foo/bar.md'), {
    scheme: 'github',
    path: 'foo/bar.md',
    format: 'md',
    version: null
  })

  t.deepEqual(split('gh://foo/bar.md@1.1.0'), {
    scheme: 'github',
    path: 'foo/bar.md',
    format: 'md',
    version: '1.1.0'
  })

  t.end()
})

test('address.scheme', t => {
  t.equal(scheme('http://foo/bar'), 'http')
  t.equal(scheme('https://foo/bar'), 'https')

  t.equal(scheme('github://foo/bar'), 'github')
  t.equal(scheme('gh://foo/bar'), 'github')

  t.end()
})

test('address.storer', t => {
  t.equal(storer('new'), null)
  t.equal(storer('local'), null)
  t.equal(storer('lib'), 'LibStorer')
  t.equal(storer('file'), 'FileStorer')
  t.equal(storer('http'), 'HttpStorer')
  t.equal(storer('https'), 'HttpStorer')
  t.equal(storer('github'), 'GithubStorer')

  t.end()
})

test('address.path', t => {
  t.equal(path('http://foo/bar'), 'foo/bar')
  t.equal(path('https://foo/bar'), 'foo/bar')

  t.end()
})

test('address.format', t => {
  t.equal(format('http://foo/bar.md'), 'md')
  t.equal(format('http://bar.html'), 'html')
  t.equal(format('http:bar.ipynb'), 'ipynb')
  t.equal(format('~/bar'), null)

  t.end()
})

test('address.version', t => {
  t.equal(version('http://foo/bar'), null)
  t.equal(version('http:foo/bar@master'), 'master')
  t.equal(version('gh:foo/bar@1.0.0'), '1.0.0')
  t.equal(version('gh:foo/bar@1.0'), '1.0')
  t.equal(version('gh:foo/bar@1'), '1')
  t.equal(version('gh:foo/bar@87bfe7e'), '87bfe7e')

  t.end()
})
