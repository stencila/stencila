/**
 * URL encoding/decoding utilities for shareable site reviews
 *
 * Encodes review items as compressed, base64url-encoded JSON in a URL parameter.
 * Uses native CompressionStream API for compression.
 */

import type { ReviewItem, SourceInfo } from './types'

/** Query parameter name for shared reviews */
export const SHARE_PARAM = '~review'

/** Size threshold to show a warning (2 KB) */
export const WARN_SIZE_BYTES = 2 * 1024

/** Maximum size before truncation (16 KB) */
export const MAX_SIZE_BYTES = 16 * 1024

/** Current format version */
const FORMAT_VERSION = 1

/**
 * Compact review payload for URL encoding (minimized keys for smaller payload)
 */
interface CompactReview {
  v: number // version
  r?: string // repository (optional)
  k?: string // commit (optional)
  i: CompactReviewItem[] // items
}

/**
 * Compact representation of ReviewItem for URL encoding
 */
interface CompactReviewItem {
  t: 'c' | 's' // type: comment | suggestion
  p: string // path (source file)
  u: string // url
  l: string // label (page title)
  s: [string, number] // start: [nodeId, offset]
  e: [string, number] // end: [nodeId, offset]
  x: string // selected text
  c: string // content
}

/**
 * Compress a string using native CompressionStream
 */
async function compress(data: string): Promise<Uint8Array> {
  const encoder = new TextEncoder()
  const stream = new ReadableStream({
    start(controller) {
      controller.enqueue(encoder.encode(data))
      controller.close()
    },
  }).pipeThrough(new CompressionStream('deflate'))

  const chunks: Uint8Array[] = []
  const reader = stream.getReader()
  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
  }

  // Concatenate chunks
  const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0)
  const result = new Uint8Array(totalLength)
  let offset = 0
  for (const chunk of chunks) {
    result.set(chunk, offset)
    offset += chunk.length
  }
  return result
}

/**
 * Decompress data using native DecompressionStream
 */
async function decompress(data: Uint8Array): Promise<string> {
  const stream = new ReadableStream({
    start(controller) {
      controller.enqueue(data)
      controller.close()
    },
  }).pipeThrough(new DecompressionStream('deflate'))

  const reader = stream.getReader()
  const decoder = new TextDecoder()
  let result = ''
  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    result += decoder.decode(value, { stream: true })
  }
  return result
}

/**
 * Encode Uint8Array to base64url (URL-safe base64)
 */
function toBase64Url(data: Uint8Array): string {
  const base64 = btoa(String.fromCharCode(...data))
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

/**
 * Decode base64url to Uint8Array
 */
function fromBase64Url(str: string): Uint8Array {
  // Restore standard base64
  let base64 = str.replace(/-/g, '+').replace(/_/g, '/')
  // Add padding if needed
  while (base64.length % 4) base64 += '='
  const binary = atob(base64)
  return Uint8Array.from(binary, (c) => c.charCodeAt(0))
}

/**
 * Convert a ReviewItem to compact format
 */
function toCompactItem(item: ReviewItem): CompactReviewItem {
  return {
    t: item.type === 'comment' ? 'c' : 's',
    p: item.path,
    u: item.url,
    l: item.title,
    s: [item.start.nodeId, item.start.offset],
    e: [item.end.nodeId, item.end.offset],
    x: item.selected,
    c: item.content,
  }
}

/**
 * Convert a compact item back to ReviewItem
 */
function fromCompactItem(compact: CompactReviewItem): ReviewItem {
  return {
    type: compact.t === 'c' ? 'comment' : 'suggestion',
    path: compact.p,
    url: compact.u,
    title: compact.l,
    start: { nodeId: compact.s[0], offset: compact.s[1] },
    end: { nodeId: compact.e[0], offset: compact.e[1] },
    selected: compact.x,
    content: compact.c,
  }
}

/**
 * Validate that data conforms to CompactReview structure
 */
function isValidCompactReview(data: unknown): data is CompactReview {
  if (typeof data !== 'object' || data === null) return false

  const obj = data as Record<string, unknown>

  // Check version
  if (typeof obj.v !== 'number') return false

  // Check items array
  if (!Array.isArray(obj.i)) return false

  // Validate each item has required fields
  for (const item of obj.i) {
    if (typeof item !== 'object' || item === null) return false
    const i = item as Record<string, unknown>
    if (typeof i.t !== 'string' || !['c', 's'].includes(i.t)) return false
    if (typeof i.p !== 'string') return false
    if (typeof i.u !== 'string') return false
    if (typeof i.l !== 'string') return false
    if (!Array.isArray(i.s) || i.s.length !== 2) return false
    if (!Array.isArray(i.e) || i.e.length !== 2) return false
    if (typeof i.x !== 'string') return false
    if (typeof i.c !== 'string') return false
  }

  return true
}

/**
 * Encode review items for URL sharing
 *
 * @param items - Review items to encode
 * @param source - Optional source info (repository, commit)
 * @returns Encoded string and metadata about truncation/warnings
 */
export async function encodeReviewForUrl(
  items: ReviewItem[],
  source?: SourceInfo
): Promise<{
  encoded: string
  includedCount: number
  truncated: boolean
  warning: boolean
}> {
  if (items.length === 0) {
    return { encoded: '', includedCount: 0, truncated: false, warning: false }
  }

  // Build compact payload
  const buildPayload = (itemsToEncode: ReviewItem[]): CompactReview => ({
    v: FORMAT_VERSION,
    ...(source?.repository && { r: source.repository }),
    ...(source?.commit && { k: source.commit }),
    i: itemsToEncode.map(toCompactItem),
  })

  // Try encoding all items first
  let payload = buildPayload(items)
  let json = JSON.stringify(payload)
  let compressed = await compress(json)
  let encoded = toBase64Url(compressed)

  // Check if we need to truncate
  if (encoded.length <= MAX_SIZE_BYTES) {
    return {
      encoded,
      includedCount: items.length,
      truncated: false,
      warning: encoded.length > WARN_SIZE_BYTES,
    }
  }

  // Binary search to find max items that fit
  let low = 1
  let high = items.length - 1
  let bestCount = 1

  while (low <= high) {
    const mid = Math.floor((low + high) / 2)
    payload = buildPayload(items.slice(0, mid))
    json = JSON.stringify(payload)
    compressed = await compress(json)
    encoded = toBase64Url(compressed)

    if (encoded.length <= MAX_SIZE_BYTES) {
      bestCount = mid
      low = mid + 1
    } else {
      high = mid - 1
    }
  }

  // Encode with best count
  payload = buildPayload(items.slice(0, bestCount))
  json = JSON.stringify(payload)
  compressed = await compress(json)
  encoded = toBase64Url(compressed)

  return {
    encoded,
    includedCount: bestCount,
    truncated: true,
    warning: true,
  }
}

/**
 * Decode a URL-encoded review
 *
 * @param encoded - The encoded string from URL parameter
 * @returns Decoded items and source, or null if invalid
 */
export async function decodeReviewFromUrl(
  encoded: string
): Promise<{ items: ReviewItem[]; source?: SourceInfo } | null> {
  if (!encoded) return null

  try {
    const compressed = fromBase64Url(encoded)
    const json = await decompress(compressed)
    const data = JSON.parse(json)

    if (!isValidCompactReview(data)) {
      console.warn('[SiteReview] Invalid shared review format')
      return null
    }

    // Handle version differences in the future
    if (data.v > FORMAT_VERSION) {
      console.warn('[SiteReview] Shared review from newer version, some data may be lost')
    }

    const items = data.i.map(fromCompactItem)
    const source: SourceInfo | undefined =
      data.r && data.k ? { repository: data.r, commit: data.k } : undefined

    return { items, source }
  } catch (e) {
    console.error('[SiteReview] Failed to decode shared review:', e)
    return null
  }
}

/**
 * Check if the current URL has a shared review parameter
 */
export function hasSharedReview(): boolean {
  const params = new URLSearchParams(window.location.search)
  return params.has(SHARE_PARAM)
}

/**
 * Extract shared review from URL and return cleaned URL
 *
 * @returns Decoded data (or null) and the URL with the parameter removed
 */
export async function extractSharedReview(): Promise<{
  data: { items: ReviewItem[]; source?: SourceInfo } | null
  cleanUrl: string
}> {
  const url = new URL(window.location.href)
  const encoded = url.searchParams.get(SHARE_PARAM)

  // Remove the parameter for clean URL
  url.searchParams.delete(SHARE_PARAM)
  const cleanUrl = url.pathname + (url.search || '') + url.hash

  if (!encoded) {
    return { data: null, cleanUrl }
  }

  const data = await decodeReviewFromUrl(encoded)
  return { data, cleanUrl }
}
