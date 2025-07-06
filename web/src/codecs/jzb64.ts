import { Entity } from '@stencila/types'

/**
 * Decode `jzb64` (JSON Zipped and Base64 encoded) into a Stencila node
 *
 * This is a browser based implementation of `from_jzb64` in
 * rust/node-url/src/lib.rs.
 */
export async function decode(jzb64: string): Promise<Entity> {
  let standardBase64 = jzb64.replace(/-/g, '+').replace(/_/g, '/')
  while (standardBase64.length % 4) {
    standardBase64 += '='
  }

  const compressedData = atob(standardBase64)

  const uint8Array = new Uint8Array(compressedData.length)
  for (let i = 0; i < compressedData.length; i++) {
    uint8Array[i] = compressedData.charCodeAt(i)
  }

  const decompressedStream = new DecompressionStream('deflate')
  const writer = decompressedStream.writable.getWriter()
  const reader = decompressedStream.readable.getReader()

  writer.write(uint8Array)
  writer.close()

  const { value } = await reader.read()
  const jsonString = new TextDecoder().decode(value)
  return JSON.parse(jsonString)
}
