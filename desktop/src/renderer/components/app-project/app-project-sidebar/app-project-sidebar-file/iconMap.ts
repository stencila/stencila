import { IconNames } from '@stencila/components'
import { File } from 'stencila'

export const getFileIcon = (
  file?: File,
  isCollapsed?: boolean,
  isMain?: boolean
): IconNames => {
  // Generic file fallback
  if (isMain) {
    return 'bookmark'
  }

  if (!file) {
    return 'file-2'
  }

  if (file.name.toLowerCase() === 'project.json') {
    return 'settings-3'
  }

  // Handle folders
  if (file.children) {
    return isCollapsed ? 'folder' : 'folder-open'
  }

  switch (file.format.name) {
    case 'csv':
      return 'layout-grid'
    case 'json':
      return 'braces'
    case 'zip':
      return 'file-zip'
    case 'r':
    case 'rmd':
    case 'ipynb':
    case 'py':
      return 'file-code'
    case 'docx':
    case 'word':
      return 'file-word'
    case 'pdf':
      return 'newspaper'
    case 'txt':
      return 'file-text'
    case 'md':
      return 'markdown'
    case 'flac':
    case 'mp3':
    case 'ogg':
      return 'mv'
    case 'gif':
    case 'jpg':
    case 'png':
      return 'image'
    case '3gp':
    case 'mp4':
    case 'ogv':
    case 'webm':
      return 'video'
    default:
      return 'file-2'
  }
}
