import { applyAdd, applyRemove, applyReplace } from '.'

test('applyAdd', () => {
  expect(applyAdd('', 0, 'a')).toEqual('a')
  expect(applyAdd('a', 1, 'e')).toEqual('ae')
  expect(applyAdd('ae', 1, 'bcd')).toEqual('abcde')
  expect(applyAdd('abcde', 2, '🏳️‍🌈')).toEqual('ab🏳️‍🌈cde')
  expect(applyAdd('ab🏳️‍🌈cde', 4, '🎁')).toEqual('ab🏳️‍🌈c🎁de')

  expect(() => applyAdd('', 'string', '')).toThrow(/Expected number slot/)
  expect(() => applyAdd('', -1, '')).toThrow(/Unexpected add slot '-1'/)
  expect(() => applyAdd('', 42, '')).toThrow(/Unexpected add slot '42'/)
})

test('applyRemove', () => {
  expect(applyRemove('ab🎁cde', 0, 1)).toEqual('b🎁cde')
  expect(applyRemove('b🎁cde', 1, 3)).toEqual('be')
  expect(applyRemove('be', 1, 1)).toEqual('b')

  expect(() => applyRemove('', 'string', 1)).toThrow(/Expected number slot/)
  expect(() => applyRemove('', -1, 1)).toThrow(/Unexpected remove slot '-1'/)
  expect(() => applyRemove('', 100, 1)).toThrow(/Unexpected remove slot '100'/)
  expect(() => applyRemove('', 0, 100)).toThrow(/Unexpected remove items 100/)
})

test('applyReplace', () => {
  expect(applyReplace('abc🎁de', 0, 1, 'x🏳️‍🌈')).toEqual('x🏳️‍🌈bc🎁de')
  expect(applyReplace('x🏳️‍🌈bc🎁de', 1, 6, 'yz')).toEqual('xyz')

  expect(() => applyReplace('', 'string', 1, '')).toThrow(
    /Expected number slot/
  )
  expect(() => applyReplace('', -1, 1, '')).toThrow(
    /Unexpected replace slot '-1'/
  )
  expect(() => applyReplace('', 42, 1, '')).toThrow(
    /Unexpected replace slot '42'/
  )
  expect(() => applyReplace('', 0, 100, '')).toThrow(
    /Unexpected replace items 100/
  )
})
