import test from 'tape'

import {shortname, longname} from '../src/language'

function check (t, lang, short, long) {
  t.equal(shortname(lang), short)
  t.equal(longname(shortname(lang)), long)
  t.equal(shortname(short), short)
  t.equal(longname(long), long)
}

test('language.{shortname|longname}', function (t) {
  check(t, 'Javascript', 'js', 'JavaScript')
  check(t, 'Julia', 'jl', 'Julia')
  check(t, 'Python', 'py', 'Python')
  check(t, 'R', 'r', 'R')
  check(t, 'SQL', 'sql', 'SQL')

  t.end()
})
