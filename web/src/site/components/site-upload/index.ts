import { html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { SiteAction, type BaseFooterState, isDevMode } from '../site-action'

import type {
  PendingFile,
  RepoFile,
  UploadAuthStatusResponse,
  UploadResponse,
} from './types'
import {
  STORAGE_KEY_FILES,
  UPLOAD_AUTH_PATH,
  UPLOAD_SUBMIT_PATH,
  UPLOAD_FILES_PATH,
  UPLOAD_CHECK_EXISTS_PATH,
  formatFileSize,
  getFileExtension,
  isExtensionAllowed,
  generateId,
  readFileAsBase64,
  getFileIcon,
  joinPath,
} from './utils'

/**
 * Site upload component
 *
 * Enables users to upload files to the repository via GitHub PRs.
 * Supports both new file uploads and updating existing files.
 */
@customElement('stencila-site-upload')
export class StencilaSiteUpload extends SiteAction<UploadAuthStatusResponse> {
  // =========================================================================
  // Abstract Method Implementations
  // =========================================================================

  get actionId() {
    return 'upload'
  }

  get actionIcon() {
    return 'i-lucide:upload'
  }

  get actionLabel() {
    return 'Upload'
  }

  get authEndpoint() {
    return UPLOAD_AUTH_PATH
  }

  get badgeCount() {
    return this.pendingFiles.length
  }

  // =========================================================================
  // Upload-Specific Properties
  // =========================================================================

  /**
   * Allowed file types (comma-separated extensions)
   */
  @property({ type: String, attribute: 'allowed-types' })
  allowedTypes: string = ''

  /**
   * Maximum file size in bytes
   */
  @property({ type: Number, attribute: 'max-size' })
  maxSize: number = 10 * 1024 * 1024 // 10MB default

  /**
   * Target path for uploads
   */
  @property({ type: String, attribute: 'target-path' })
  targetPath: string = ''

  /**
   * Whether users can specify custom paths
   */
  @property({ type: Boolean, attribute: 'user-path' })
  userPath: boolean = false

  /**
   * Whether overwriting existing files is allowed
   */
  @property({ type: Boolean, attribute: 'allow-overwrite' })
  allowOverwrite: boolean = true

  /**
   * Whether a message is required
   */
  @property({ type: Boolean, attribute: 'require-message' })
  requireMessage: boolean = false

  /**
   * Get allowed types as array
   */
  private get allowedTypesArray(): string[] | null {
    if (!this.allowedTypes) return null
    return this.allowedTypes.split(',').map((t) => t.trim().toLowerCase())
  }

  // =========================================================================
  // Upload-Specific State
  // =========================================================================

  @state()
  private activeTab: 'upload' | 'update' = 'upload'

  @state()
  private pendingFiles: PendingFile[] = []

  @state()
  private repoFiles: RepoFile[] = []

  @state()
  private repoFilesLoading: boolean = false

  @state()
  private selectedRepoFile: RepoFile | null = null

  @state()
  private message: string = ''

  @state()
  private isDragOver: boolean = false

  // Note: authStatus, errorMessage are inherited from SiteAction base class

  // Lifecycle

  override connectedCallback() {
    super.connectedCallback()
    this.loadPendingFiles()
    // Note: fetchAuthStatus() is called by base class
  }

  // Note: openPanel(), closePanel() are inherited from SiteAction base class

  // Storage

  private loadPendingFiles() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY_FILES)
      if (stored) {
        this.pendingFiles = JSON.parse(stored)
      }
    } catch {
      // Ignore storage errors
    }
  }

  private savePendingFiles() {
    try {
      localStorage.setItem(STORAGE_KEY_FILES, JSON.stringify(this.pendingFiles))
    } catch {
      // Ignore storage errors
    }
  }

  private clearPendingFiles() {
    this.pendingFiles = []
    localStorage.removeItem(STORAGE_KEY_FILES)
  }

  // =========================================================================
  // Abstract Method Implementations - Auth
  // =========================================================================

  /**
   * Apply development defaults when on localhost without API
   */
  protected override applyDevDefaults(): void {
    this.authStatus = {
      hasSiteAccess: true,
      user: { id: 'dev', name: 'Dev User', avatar: '' },
      github: {
        connected: true,
        username: 'dev-user',
        canPush: true,
        source: 'oauth',
      },
      uploadConfig: {
        enabled: true,
        allowPublic: true,
        allowAnonymous: false,
        allowedTypes: this.allowedTypesArray,
        maxSize: this.maxSize,
        targetPath: this.targetPath,
        userPath: this.userPath,
        allowOverwrite: this.allowOverwrite,
        requireMessage: this.requireMessage,
      },
      repo: { isPrivate: false, appInstalled: true },
      authorship: { canAuthorAsSelf: true, willBeBotAuthored: false },
    }
  }

  /**
   * Calculate the current footer state based on auth and submission status
   */
  protected override calculateFooterState(): BaseFooterState {
    // Check submission states first
    if (this.isSubmitting) {
      return { type: 'submitting' }
    }

    if (this.submittedPr) {
      return {
        type: 'success',
        prNumber: this.submittedPr.number,
        prUrl: this.submittedPr.url,
      }
    }

    if (this.authLoading || !this.authStatus) {
      return { type: 'loading' }
    }

    if (!this.authStatus.uploadConfig?.enabled) {
      return { type: 'blocked', reason: 'Uploads are disabled' }
    }

    if (!this.authStatus.hasSiteAccess) {
      return { type: 'needSiteAccess', signInUrl: this.signInUrl }
    }

    if (
      !this.authStatus.uploadConfig.allowAnonymous &&
      !this.authStatus.github?.connected
    ) {
      if (!this.authStatus.user) {
        return { type: 'needStencilaSignIn', signInUrl: this.signInUrl }
      }
      return { type: 'needGitHubConnect' }
    }

    return {
      type: 'canSubmit',
      authorDescription: this.getAuthorDescription(),
    }
  }

  private getAuthorDescription(): string {
    if (!this.authStatus) return ''

    if (this.authStatus.authorship?.willBeBotAuthored) {
      if (this.authStatus.user) {
        return `PR will be created by Stencila bot on behalf of ${this.authStatus.user.name}`
      }
      return 'PR will be created by Stencila bot'
    }

    if (this.authStatus.github?.connected) {
      return `PR will be created as ${this.authStatus.github.username}`
    }

    return ''
  }

  // File handling

  private async handleFileDrop(e: DragEvent) {
    e.preventDefault()
    this.isDragOver = false

    const files = e.dataTransfer?.files
    if (!files || files.length === 0) return

    await this.processFiles(Array.from(files))
  }

  private async handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement
    const files = input.files
    if (!files || files.length === 0) return

    await this.processFiles(Array.from(files))
    input.value = '' // Reset input
  }

  private async processFiles(files: File[]) {
    for (const file of files) {
      // Validate extension
      if (!isExtensionAllowed(file.name, this.allowedTypesArray)) {
        this.errorMessage = `File type not allowed: ${getFileExtension(file.name)}`
        continue
      }

      // Validate size
      if (file.size > this.maxSize) {
        this.errorMessage = `File too large: ${formatFileSize(file.size)} (max ${formatFileSize(this.maxSize)})`
        continue
      }

      // Read file content
      try {
        const content = await readFileAsBase64(file)
        const targetPath = this.targetPath
          ? joinPath(this.targetPath, file.name)
          : file.name

        // Check if overwriting
        const isOverwrite = await this.checkFileExists(targetPath)

        if (isOverwrite && !this.allowOverwrite) {
          this.errorMessage = `Cannot overwrite existing file: ${targetPath}`
          continue
        }

        const pendingFile: PendingFile = {
          id: generateId(),
          filename: file.name,
          targetPath,
          size: file.size,
          mimeType: file.type,
          isOverwrite,
          content,
        }

        this.pendingFiles = [...this.pendingFiles, pendingFile]
        this.savePendingFiles()
      } catch (_err) {
        this.errorMessage = `Failed to read file: ${file.name}`
      }
    }
  }

  private async checkFileExists(path: string): Promise<boolean> {
    if (isDevMode(this.actionId)) return false

    try {
      const response = await this.apiFetch(
        `${UPLOAD_CHECK_EXISTS_PATH}?path=${encodeURIComponent(path)}`
      )
      if (response.ok) {
        const data = await response.json()
        return data.exists
      }
    } catch {
      // Assume file doesn't exist on error
    }
    return false
  }

  private removeFile(id: string) {
    this.pendingFiles = this.pendingFiles.filter((f) => f.id !== id)
    this.savePendingFiles()
  }

  private updateFilePath(id: string, newPath: string) {
    this.pendingFiles = this.pendingFiles.map((f) =>
      f.id === id ? { ...f, targetPath: newPath } : f
    )
    this.savePendingFiles()
  }

  // Repo file browser (for update mode)

  private async fetchRepoFiles() {
    if (isDevMode(this.actionId)) {
      this.repoFiles = [
        { path: 'data/sample.csv', size: 1234, lastModified: '2024-01-15' },
        { path: 'data/config.json', size: 567, lastModified: '2024-01-10' },
      ]
      return
    }

    this.repoFilesLoading = true
    try {
      const params = new URLSearchParams()
      if (this.allowedTypesArray) {
        params.set('extensions', this.allowedTypesArray.join(','))
      }
      if (this.targetPath) {
        params.set('path', this.targetPath)
      }

      const response = await this.apiFetch(`${UPLOAD_FILES_PATH}?${params}`)
      if (response.ok) {
        const data = await response.json()
        this.repoFiles = data.files || []
      }
    } catch {
      // Failed to load files
    }
    this.repoFilesLoading = false
  }

  private selectRepoFile(file: RepoFile) {
    this.selectedRepoFile = file
  }

  private async handleUpdateFile(e: DragEvent) {
    e.preventDefault()
    this.isDragOver = false

    if (!this.selectedRepoFile) return

    const files = e.dataTransfer?.files
    if (!files || files.length !== 1) {
      this.errorMessage = 'Please drop exactly one file to replace'
      return
    }

    const file = files[0]

    // Validate size
    if (file.size > this.maxSize) {
      this.errorMessage = `File too large: ${formatFileSize(file.size)} (max ${formatFileSize(this.maxSize)})`
      return
    }

    try {
      const content = await readFileAsBase64(file)

      const pendingFile: PendingFile = {
        id: generateId(),
        filename: file.name,
        targetPath: this.selectedRepoFile.path,
        size: file.size,
        mimeType: file.type,
        isOverwrite: true,
        content,
      }

      this.pendingFiles = [...this.pendingFiles, pendingFile]
      this.savePendingFiles()
      this.selectedRepoFile = null
    } catch {
      this.errorMessage = 'Failed to read file'
    }
  }

  // Submission

  /**
   * Track submission state locally (not in base class footerState)
   */
  @state()
  private isSubmitting: boolean = false

  @state()
  private submittedPr: { number: number; url: string } | null = null

  private async handleSubmit() {
    if (this.pendingFiles.length === 0) return

    if (this.requireMessage && !this.message.trim()) {
      this.errorMessage = 'Please enter a description'
      return
    }

    this.isSubmitting = true
    this.submittedPr = null

    if (isDevMode(this.actionId)) {
      // Simulate upload in dev mode
      await new Promise((resolve) => setTimeout(resolve, 1000))
      this.submittedPr = {
        number: 123,
        url: 'https://github.com/example/repo/pull/123',
      }
      this.isSubmitting = false
      this.clearPendingFiles()
      return
    }

    try {
      const response = await this.apiFetch(UPLOAD_SUBMIT_PATH, {
        method: 'POST',
        body: {
          files: this.pendingFiles.map((f) => ({
            filename: f.filename,
            targetPath: f.targetPath,
            content: f.content,
            overwrite: f.isOverwrite,
          })),
          message: this.message || 'Upload files via Stencila',
          authorAsSelf: !this.authStatus?.authorship?.willBeBotAuthored,
        },
      })

      if (response.ok) {
        const data: UploadResponse = await response.json()
        this.submittedPr = { number: data.prNumber, url: data.prUrl }
        this.clearPendingFiles()
        this.message = ''
      } else {
        const error = await response.json()
        this.errorMessage = error.message || 'Upload failed'
      }
    } catch {
      this.errorMessage = 'Network error'
    } finally {
      this.isSubmitting = false
    }
  }

  // =========================================================================
  // Rendering
  // =========================================================================

  /**
   * Main render method - uses base class FAB, panel, and error modal
   */
  override render() {
    return html`
      ${this.renderFab()}
      ${this.renderPanel()}
      ${this.renderErrorModal()}
    `
  }

  /**
   * Render upload-specific panel content (abstract method implementation)
   */
  protected override renderPanelContent() {
    return html`
      <!-- Tabs -->
      <div class="tabs">
        <button
          class="tab ${this.activeTab === 'upload' ? 'active' : ''}"
          @click=${() => (this.activeTab = 'upload')}
        >
          Upload New
        </button>
        <button
          class="tab ${this.activeTab === 'update' ? 'active' : ''}"
          @click=${() => {
            this.activeTab = 'update'
            if (this.repoFiles.length === 0) {
              this.fetchRepoFiles()
            }
          }}
        >
          Update Existing
        </button>
      </div>

      <!-- Tab Content -->
      <div class="content">
        ${this.activeTab === 'upload'
          ? this.renderUploadTab()
          : this.renderUpdateTab()}
      </div>

      <!-- Pending Files -->
      ${this.renderPendingFiles()}

      <!-- Message Input -->
      ${this.renderMessageInput()}

      <!-- Footer with status and submit button -->
      <div class="site-action-panel-footer">
        ${this.renderFooterStatus()}
        ${this.renderSubmitButton()}
      </div>
    `
  }

  private renderUploadTab() {
    return html`
      <div
        class="dropzone ${this.isDragOver ? 'drag-over' : ''}"
        @dragover=${(e: DragEvent) => {
          e.preventDefault()
          this.isDragOver = true
        }}
        @dragleave=${() => (this.isDragOver = false)}
        @drop=${this.handleFileDrop}
      >
        <div class="dropzone-content">
          <span class="i-lucide:upload dropzone-icon"></span>
          <p>Drop files here or click to browse</p>
          ${this.allowedTypesArray
            ? html`<p class="hint">
                Allowed: ${this.allowedTypesArray.join(', ')}
              </p>`
            : nothing}
          <p class="hint">Max size: ${formatFileSize(this.maxSize)}</p>
          <input
            type="file"
            multiple
            @change=${this.handleFileSelect}
            accept=${this.allowedTypesArray
              ? this.allowedTypesArray.map((t) => `.${t}`).join(',')
              : ''}
          />
        </div>
      </div>
    `
  }

  private renderUpdateTab() {
    if (this.repoFilesLoading) {
      return html`<div class="loading">Loading files...</div>`
    }

    if (this.repoFiles.length === 0) {
      return html`<div class="empty">No files found in repository</div>`
    }

    return html`
      <div class="file-browser">
        <div class="file-list">
          ${this.repoFiles.map(
            (file) => html`
              <div
                class="file-item ${this.selectedRepoFile?.path === file.path
                  ? 'selected'
                  : ''}"
                @click=${() => this.selectRepoFile(file)}
              >
                <span class="file-icon">${getFileIcon(file.path)}</span>
                <span class="file-path">${file.path}</span>
                <span class="file-size">${formatFileSize(file.size)}</span>
              </div>
            `
          )}
        </div>

        ${this.selectedRepoFile
          ? html`
              <div
                class="dropzone update-dropzone ${this.isDragOver
                  ? 'drag-over'
                  : ''}"
                @dragover=${(e: DragEvent) => {
                  e.preventDefault()
                  this.isDragOver = true
                }}
                @dragleave=${() => (this.isDragOver = false)}
                @drop=${this.handleUpdateFile}
              >
                <p>Drop replacement file for:</p>
                <strong>${this.selectedRepoFile.path}</strong>
              </div>
            `
          : html`<p class="hint">Select a file to replace</p>`}
      </div>
    `
  }

  private renderPendingFiles() {
    if (this.pendingFiles.length === 0) return nothing

    return html`
      <div class="pending">
        <h4 class="pending-header">Pending Uploads (${this.pendingFiles.length})</h4>
        <ul class="pending-list">
          ${this.pendingFiles.map(
            (file) => html`
              <li class="pending-file">
                <div class="file-info">
                  <span class="file-icon">${getFileIcon(file.filename)}</span>
                  <span class="file-name">${file.filename}</span>
                  ${file.isOverwrite
                    ? html`<span class="overwrite-badge">update</span>`
                    : nothing}
                </div>
                <div class="file-path">
                  ${this.userPath
                    ? html`<input
                        type="text"
                        class="path-input"
                        .value=${file.targetPath}
                        @change=${(e: Event) =>
                          this.updateFilePath(
                            file.id,
                            (e.target as HTMLInputElement).value
                          )}
                      />`
                    : html`<span>${file.targetPath}</span>`}
                </div>
                <div class="file-size">${formatFileSize(file.size)}</div>
                <button
                  class="remove-btn"
                  @click=${() => this.removeFile(file.id)}
                  aria-label="Remove file"
                >
                  <span class="i-lucide:x"></span>
                </button>
              </li>
            `
          )}
        </ul>
      </div>
    `
  }

  private renderMessageInput() {
    return html`
      <div class="message">
        <label class="message-label">
          Description ${this.requireMessage ? html`<span class="required">*</span>` : nothing}
        </label>
        <textarea
          class="message-input"
          placeholder="Describe your changes..."
          .value=${this.message}
          @input=${(e: Event) =>
            (this.message = (e.target as HTMLTextAreaElement).value)}
        ></textarea>
      </div>
    `
  }

  /**
   * Render the submit button with file count
   */
  private renderSubmitButton() {
    const state = this.calculateFooterState()
    const fileCount = this.pendingFiles.length

    // Only show submit button when user can submit
    if (state.type !== 'canSubmit') {
      return nothing
    }

    return html`
      <button
        class="site-action-btn primary"
        ?disabled=${fileCount === 0 || this.isSubmitting}
        @click=${this.handleSubmit}
      >
        ${this.isSubmitting
          ? 'Creating PR...'
          : `Create PR (${fileCount} file${fileCount !== 1 ? 's' : ''})`}
      </button>
    `
  }
}
