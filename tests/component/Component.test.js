const os = require('os')

const test = require('tape')

const Component = require('../../src/component/Component')

const ComponentConverterUnknown = require('../../src/component/component-converter-errors').ComponentConverterUnknown
const ComponentStorerUnknown = require('../../src/component/component-storer-errors').ComponentStorerUnknown

const ComponentGithubStorer = require('../../src/component/ComponentGithubStorer')
const ComponentHttpStorer = require('../../src/component/ComponentHttpStorer')

test('Component.long', t => {
  t.equal(Component.long('new://document'), 'new://document')
  t.equal(Component.long('+document'), 'new://document')
  t.equal(Component.long('*aaaaaaaa'), 'name://aaaaaaaa')

  t.equal(Component.long('file:///report.md'), 'file:///report.md')
  t.equal(Component.long('file:/report.md'), 'file:///report.md')
  t.equal(Component.long('file:report.md'), 'file:///report.md')
  t.equal(Component.long('./report.docx'), 'file://' + process.cwd() + '/report.docx')
  t.equal(Component.long('/some/dir/report.docx'), 'file:///some/dir/report.docx')
  t.equal(Component.long('~/report.docx'), 'file://' + os.homedir() + '/report.docx')

  t.equal(Component.long('http:foo.com/report.md'), 'http://foo.com/report.md')
  t.equal(Component.long('https://foo.com/report.md'), 'https://foo.com/report.md')

  t.equal(Component.long('github:/user/repo/report.md'), 'gh://user/repo/report.md')
  t.equal(Component.long('gh:/user/repo/report.md'), 'gh://user/repo/report.md')

  t.equal(Component.long('stats/t-test'), 'st://stats/t-test')
  t.end()
})

test('Component.long on instance', t => {
  let c = new Component('./report.docx')
  t.equal(c.long(), Component.long(c.address))
  t.end()
})

test('Component.short', t => {
  t.equal(Component.short('new://document'), '+document')
  t.equal(Component.short('name://aaaaaaaa'), '*aaaaaaaa')
  t.equal(Component.short('file://report.docx'), 'file:report.docx')
  t.equal(Component.short('https://foo.com/report.md'), 'https:foo.com/report.md')
  t.equal(Component.short('gh://foo/bar/report.md'), 'gh:foo/bar/report.md')
  t.equal(Component.short('st://stats/t-test'), 'stats/t-test')
  t.end()
})

test('Component.short on instance', t => {
  let c = new Component('file:///report.docx')
  t.equal(c.short(), Component.short(c.address))
  t.end()
})

test('Component.short+long', t => {
  let f = (address) => {
    return Component.short(Component.long(address))
  }
  t.equal(f('+document'), '+document')
  t.equal(f('new://document'), '+document')
  t.equal(f('*aaaaaaaa'), '*aaaaaaaa')
  t.equal(f('name://aaaaaaaa'), '*aaaaaaaa')
  t.equal(f('gh:foo/bar/report.md'), 'gh:foo/bar/report.md')
  t.equal(f('gh:foo/bar/report.md@1.1.0'), 'gh:foo/bar/report.md@1.1.0')
  t.end()
})

test('Component.split', t => {
  t.deepEqual(Component.split('+document'), {
    scheme: 'new',
    path: 'document',
    format: null,
    version: null
  })

  t.deepEqual(Component.split('*aaaaaaaa'), {
    scheme: 'name',
    path: 'aaaaaaaa',
    format: null,
    version: null
  })

  t.deepEqual(Component.split('stats/t-test'), {
    scheme: 'st',
    path: 'stats/t-test',
    format: null,
    version: null
  })

  t.deepEqual(Component.split('stats/t-test@1.1.0'), {
    scheme: 'st',
    path: 'stats/t-test',
    format: null,
    version: '1.1.0'
  })

  t.deepEqual(Component.split('http://foo/bar'), {
    scheme: 'http',
    path: 'foo/bar',
    format: null,
    version: null
  })

  t.deepEqual(Component.split('gh://foo/bar.md'), {
    scheme: 'gh',
    path: 'foo/bar.md',
    format: 'md',
    version: null
  })

  t.deepEqual(Component.split('gh://foo/bar.md@1.1.0'), {
    scheme: 'gh',
    path: 'foo/bar.md',
    format: 'md',
    version: '1.1.0'
  })

  t.end()
})

test('Component.split on instance', t => {
  let c = new Component('gh://user/repo/README.md')
  t.deepEqual(c.split(), Component.split(c.address))
  t.end()
})

test('Component.scheme', t => {
  t.equal(Component.scheme('http://foo/bar'), 'http')
  t.equal(Component.scheme('https://foo/bar'), 'https')

  t.equal(Component.scheme('github://foo/bar'), 'gh')
  t.equal(Component.scheme('gh://foo/bar'), 'gh')

  t.end()
})

test('Component.default', t => {
  let c = new Component()
  t.equal(Component.default('foo'), null)
  t.equal(c.default('foo'), Component.default('foo'))
  t.end()
})

test('Component.converter', t => {
  t.throws(() => {
    Component.converter('foo')
  }, ComponentConverterUnknown)
  t.end()
})

test('Component.storer', t => {
  t.ok((new Component('http://foo/bar')).storer() instanceof ComponentHttpStorer)
  t.ok((new Component('https://foo/bar')).storer() instanceof ComponentHttpStorer)
  t.ok((new Component('gh://foo/bar')).storer() instanceof ComponentGithubStorer)
  t.throws(() => {
    Component.storer('foo')
  }, ComponentStorerUnknown)
  t.end()
})
