import manifest from '../src/manifest'

test('manifest', () => {
  const mani = manifest()
  expect(mani.stencila).toBeTruthy()
  expect(mani.services.import).toEqual(['application/ld+json'])
  expect(mani.services.export).toEqual(['application/ld+json'])
  expect(mani.services.compile).toEqual([])
  expect(mani.services.build).toEqual([])
  expect(mani.services.execute).toEqual([])
})
