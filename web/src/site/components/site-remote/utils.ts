/**
 * Constants and utility functions for site remote components
 */

// Storage keys
export const STORAGE_KEY_PENDING = 'stencila-site-remote-pending'

// API endpoint paths (relative, will be prefixed with apiBase)
export const REMOTE_SUBMIT_PATH = '/__stencila/remotes'

// Picker URLs
export const GOOGLE_PICKER_URL = 'https://stencila.cloud/google/picker'
export const MICROSOFT_PICKER_URL = 'https://stencila.cloud/microsoft/picker'

// Expected origin for picker postMessage
export const PICKER_ORIGIN = 'https://stencila.cloud'

// Polling interval for picker close detection (ms)
export const PICKER_POLL_INTERVAL = 500

/**
 * Format options with labels
 *
 * Values match backend config (without dots): "smd", "md", "html"
 * Labels show the file extension for user clarity
 */
export const FORMAT_OPTIONS = [
  { value: 'smd', label: 'Stencila Markdown (.smd)' },
  { value: 'md', label: 'Markdown (.md)' },
  { value: 'html', label: 'HTML (.html)' },
] as const

/**
 * Get the file extension for a format (adds dot prefix)
 */
export function formatToExtension(format: string): string {
  return `.${format}`
}

/**
 * Extract format from file path extension
 * Returns 'smd' as default if extension is not recognized
 */
export function getFormatFromPath(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase()
  if (ext === 'md') return 'md'
  if (ext === 'html') return 'html'
  return 'smd'
}

/**
 * Get sync direction options with dynamic service name
 * Bi-directional is first as the recommended default
 */
export function getSyncDirectionOptions(serviceName: string) {
  return [
    { value: 'bi', label: 'Bi-directional', description: `${serviceName} ↔ Site` },
    { value: 'from-remote', label: 'From remote', description: `${serviceName} → Site` },
    { value: 'to-remote', label: 'To remote', description: `Site → ${serviceName}` },
  ] as const
}

/**
 * Common document extensions to strip from titles
 */
const DOCUMENT_EXTENSIONS = [
  '.docx', '.doc', '.xlsx', '.xls', '.pptx', '.ppt',
  '.odt', '.ods', '.odp', '.pdf', '.rtf', '.txt'
]

/**
 * Strip common document extensions from a filename/title
 */
export function stripDocumentExtension(title: string): string {
  const lower = title.toLowerCase()
  for (const ext of DOCUMENT_EXTENSIONS) {
    if (lower.endsWith(ext)) {
      return title.slice(0, -ext.length)
    }
  }
  return title
}

/**
 * Generate a slug from a document title for use in file paths
 */
export function slugify(title: string): string {
  // First strip any document extension
  const titleWithoutExt = stripDocumentExtension(title)

  return titleWithoutExt
    .toLowerCase()
    .trim()
    .replace(/\./g, '-') // Convert dots to hyphens
    .replace(/[^\w\s-]/g, '') // Remove non-word chars except spaces and hyphens
    .replace(/\s+/g, '-') // Replace spaces with hyphens
    .replace(/-+/g, '-') // Replace multiple hyphens with single
    .replace(/^-|-$/g, '') // Remove leading/trailing hyphens
    .substring(0, 50) // Limit length
}

/**
 * Generate a target path from document title and format
 *
 * @param title - Document title to slugify
 * @param format - Format without dot (e.g., "smd", "md", "html")
 * @param targetDir - Optional target directory
 */
export function generateTargetPath(
  title: string,
  format: string,
  targetDir: string
): string {
  const slug = slugify(title)
  const filename = slug + formatToExtension(format)
  return targetDir ? joinPath(targetDir, filename) : filename
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

/**
 * Get display name for a service
 */
export function getServiceDisplayName(service: 'gdoc' | 'm365'): string {
  return service === 'gdoc' ? 'Google Docs' : 'Microsoft 365'
}

/**
 * Validate that a URL is a valid Google Docs or M365 URL
 */
export function isValidRemoteUrl(url: string): boolean {
  try {
    const parsed = new URL(url)
    // Google Docs
    if (
      parsed.hostname === 'docs.google.com' &&
      parsed.pathname.includes('/document/')
    ) {
      return true
    }
    // Microsoft 365 (OneDrive, SharePoint)
    if (
      parsed.hostname.includes('sharepoint.com') ||
      parsed.hostname === 'onedrive.live.com' ||
      parsed.hostname.includes('1drv.ms')
    ) {
      return true
    }
    return false
  } catch {
    return false
  }
}
