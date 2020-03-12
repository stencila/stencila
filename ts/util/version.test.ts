import pkg from '../../package.json'
import { version, versionMajor, versionMinor } from './version'

test('version', () => {
  expect(version).toEqual(pkg.version)
})

test('versionMajor', () => {
  expect(versionMajor).toEqual(pkg.version.split('.')[0])
})

test('versionMinor', () => {
  expect(versionMinor).toEqual(
    pkg.version
      .split('.')
      .slice(0, 2)
      .join('.')
  )
})
