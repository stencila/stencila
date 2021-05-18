import { IconNames } from '@stencila/components'
import { File } from 'stencila'

export const getFileIcon = (file?: File): IconNames => {
  // Generic file fallback
  if (!file) {
    return 'file-2'
  }

  // Handle folders
  if (file.children) {
    return 'folder'
  }

  // Don't differentiate between image formats for now
  if (file.mediaType?.includes('image')) {
    return 'image'
  }

  switch (file.format) {
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
      return 'article'
    case 'txt':
      return 'file-text'
    default:
      return 'file-2'
  }
}
