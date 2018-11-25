import OperatingSystem from '../src/types/OperatingSystem'

test('type', () => {
  const operatingSystem = new OperatingSystem()
  expect(operatingSystem.type).toEqual('OperatingSystem')
})

test('instances', () => {
  expect(OperatingSystem.linux.name).toEqual('Linux')
  expect(OperatingSystem.macos.name).toEqual('macOS')
  expect(OperatingSystem.windows.name).toEqual('Windows')
})
