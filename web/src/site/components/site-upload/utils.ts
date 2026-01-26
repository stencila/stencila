/**
 * Constants and utility functions for site upload components
 */

// Storage keys
export const STORAGE_KEY_FILES = 'stencila-site-upload-files'

// API endpoint paths (relative, will be prefixed with apiBase)
export const UPLOAD_SUBMIT_PATH = '/__stencila/uploads'
export const UPLOAD_FILES_PATH = '/__stencila/uploads/files'
export const UPLOAD_CHECK_EXISTS_PATH = '/__stencila/uploads/check-exists'

// External URLs
export const GITHUB_OAUTH_URL = 'https://stencila.cloud/github/connect/site'

/**
 * Check if running on localhost
 */
export function isLocalhost(): boolean {
  const hostname = window.location.hostname
  return hostname === 'localhost' || hostname === '127.0.0.1'
}

/**
 * Check if this is a Stencila-hosted site (*.stencila.site or localhost)
 */
export function isStencilaHostedSite(): boolean {
  const hostname = window.location.hostname
  if (hostname.endsWith('.stencila.site')) return true
  if (hostname === 'localhost' || hostname === '127.0.0.1') return true
  const thirdPartyHosts = [
    'netlify.app',
    'netlify.com',
    'github.io',
    'vercel.app',
    'pages.dev',
    'surge.sh',
    'render.com',
  ]
  return !thirdPartyHosts.some((host) => hostname.endsWith(host))
}

/**
 * Check if dev mode is enabled
 */
export function isDevMode(): boolean {
  if (!isLocalhost()) return false
  const urlParams = new URLSearchParams(window.location.search)
  if (urlParams.get('uploadDevMode') === 'true') return true
  if (localStorage.getItem('stencila-upload-dev-mode') === 'true') return true
  return false
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

/**
 * Get file extension from filename
 */
export function getFileExtension(filename: string): string {
  const lastDot = filename.lastIndexOf('.')
  if (lastDot === -1) return ''
  return filename.slice(lastDot + 1).toLowerCase()
}

/**
 * Check if a file extension is allowed.
 * Handles extensions with or without leading dots (e.g., both "csv" and ".csv").
 */
export function isExtensionAllowed(
  filename: string,
  allowedTypes: string[] | null
): boolean {
  if (!allowedTypes || allowedTypes.length === 0) return true
  const ext = getFileExtension(filename).toLowerCase()
  // Normalize allowed types by removing leading dots and lowercasing
  return allowedTypes.some((t) => t.replace(/^\./, '').toLowerCase() === ext)
}

/**
 * Generate a unique ID
 */
export function generateId(): string {
  return Math.random().toString(36).substring(2, 11)
}

/**
 * Read file as base64
 */
export function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      // Remove the data URL prefix (e.g., "data:text/csv;base64,")
      const base64 = result.split(',')[1]
      resolve(base64)
    }
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

/**
 * Get icon name for file type
 */
export function getFileIcon(filename: string): string {
  const ext = getFileExtension(filename)
  const iconMap: Record<string, string> = {
    csv: 'table',
    json: 'code',
    xlsx: 'table',
    xls: 'table',
    md: 'file-text',
    txt: 'file-text',
    pdf: 'file',
    png: 'image',
    jpg: 'image',
    jpeg: 'image',
    gif: 'image',
    svg: 'image',
  }
  return iconMap[ext] || 'file'
}

/**
 * Join path segments, handling leading/trailing slashes
 */
export function joinPath(...segments: string[]): string {
  return segments
    .map((s) => s.replace(/^\/+|\/+$/g, ''))
    .filter((s) => s.length > 0)
    .join('/')
}
