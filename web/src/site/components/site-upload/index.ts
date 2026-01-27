import { html, nothing } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { GlideEvents } from '../../glide/events'
import { SiteAction, type BaseFooterState, isLocalhost, joinPath } from '../site-action'
import { FilesIndexLoader } from '../site-files'

import type {
  PendingFile,
  RepoFile,
  UploadResponse,
} from './types'
import {
  STORAGE_KEY_FILES,
  UPLOAD_SUBMIT_PATH,
  formatFileSize,
  getFileExtension,
  isExtensionAllowed,
  generateId,
  readFileAsBase64,
  getFileIcon,
} from './utils'

/**
 * Site upload component
 *
 * Enables users to upload files to the repository via GitHub PRs.
 * Supports both new file uploads and updating existing files.
 */
@customElement('stencila-site-upload')
export class StencilaSiteUpload extends SiteAction {
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

  get badgeCount() {
    return this.pendingFiles.length
  }

  get isActionAllowed() {
    const config = this.authStatus?.uploadConfig
    return config?.enabled === true && config?.allowed === true
  }

  // =========================================================================
  // Upload-Specific Computed Properties
  // =========================================================================

  /**
   * Default max file size for dev mode (10MB)
   */
  private static readonly DEFAULT_MAX_FILE_SIZE = 10 * 1024 * 1024

  /**
   * Normalize an extension by removing leading dot and lowercasing.
   * e.g., ".CSV" -> "csv", "csv" -> "csv"
   */
  private normalizeExtension(ext: string): string {
    return ext.replace(/^\./, '').toLowerCase()
  }

  /**
   * Get effective allowed extensions from server config.
   * Extensions are normalized (no leading dot, lowercase).
   *
   * Returns `null` when any extension is allowed (server returned null or []).
   */
  private get effectiveAllowedExtensions(): string[] | null {
    const serverConfig = this.authStatus?.uploadConfig?.allowedExtensions
    // null or empty array means any extension is allowed
    if (serverConfig === null || serverConfig === undefined || serverConfig.length === 0) {
      return null
    }
    // Normalize extensions (remove leading dots, lowercase)
    return serverConfig.map((ext) => this.normalizeExtension(ext))
  }

  /**
   * Get effective max file size from server config.
   */
  private get effectiveMaxFileSize(): number {
    return this.authStatus?.uploadConfig?.maxFileSize ?? StencilaSiteUpload.DEFAULT_MAX_FILE_SIZE
  }

  /**
   * Get the path attribute from the closest root element.
   * This is the source file path for the current page.
   */
  private getPathFromRoot(): string | null {
    const root = document.querySelector('[root]')
    return root?.getAttribute('path') ?? null
  }

  /**
   * Get target path derived from source file path (parent directory).
   * e.g., docs/guide/intro.smd → docs/guide
   */
  private get effectiveTargetPath(): string {
    const sourcePath = this.getPathFromRoot()
    if (!sourcePath) {
      return '' // Root level if no path found
    }
    // Get parent directory
    const lastSlash = sourcePath.lastIndexOf('/')
    if (lastSlash > 0) {
      return sourcePath.substring(0, lastSlash)
    }
    return '' // Root level pages
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

  /**
   * Loader for files index (used to browse existing files)
   */
  private filesLoader = new FilesIndexLoader()

  @state()
  private selectedRepoFile: RepoFile | null = null

  @state()
  private message: string = ''

  @state()
  private isDragOver: boolean = false

  @state()
  private fileSearchQuery: string = ''

  // Note: authStatus, errorMessage are inherited from SiteAction base class

  // Lifecycle

  override connectedCallback() {
    super.connectedCallback()
    this.loadPendingFiles()
    // Note: fetchAuthStatus() is called by base class

    // Listen for client-side navigation to refresh file list
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  /**
   * Handle Glide navigation end - clear and refresh file list for new page
   */
  private handleGlideEnd = () => {
    // Clear current file list, selection, and search
    this.repoFiles = []
    this.selectedRepoFile = null
    this.fileSearchQuery = ''

    // Clear the files loader cache for this directory since we navigated away
    this.filesLoader.clearCache()

    // If the Update Existing tab is active, fetch files for the new page
    if (this.activeTab === 'update') {
      this.fetchRepoFiles()
    }
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
   * Apply permissive defaults for localhost preview
   */
  protected override applyPreviewDefaults(): void {
    this.authStatus = {
      hasSiteAccess: true,
      user: { id: 'preview', name: 'Preview User', avatar: '' },
      github: {
        connected: true,
        username: 'preview-user',
        canPush: true,
        source: 'oauth',
      },
      uploadConfig: {
        enabled: true,
        allowed: true,
        allowedDirectories: null,
        maxFileSize: StencilaSiteUpload.DEFAULT_MAX_FILE_SIZE,
        allowedExtensions: null,
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
      return { type: 'blocked', reason: 'Uploads are disabled for this site' }
    }

    // Use component attribute for public access (not server config)
    if (!this.public && !this.authStatus.hasSiteAccess) {
      return { type: 'needSiteAccess', signInUrl: this.signInUrl }
    }

    // If anonymous submissions not allowed, require GitHub connection
    if (!this.anon && !this.authStatus.github?.connected) {
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
      // Validate extension (if restrictions apply)
      if (!isExtensionAllowed(file.name, this.effectiveAllowedExtensions)) {
        this.errorMessage = `File type not allowed: ${getFileExtension(file.name)}`
        continue
      }

      // Validate size
      if (file.size > this.effectiveMaxFileSize) {
        this.errorMessage = `File too large: ${formatFileSize(file.size)} (max ${formatFileSize(this.effectiveMaxFileSize)})`
        continue
      }

      // Read file content
      try {
        const content = await readFileAsBase64(file)
        const targetPath = this.effectiveTargetPath
          ? joinPath(this.effectiveTargetPath, file.name)
          : file.name

        // Check if overwriting (always allowed, but track for display)
        const isOverwrite = await this.checkFileExists(targetPath)

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
        // Clear previous PR success state when adding new files
        this.submittedPr = null
      } catch (_err) {
        this.errorMessage = `Failed to read file: ${file.name}`
      }
    }
  }

  private async checkFileExists(path: string): Promise<boolean> {
    if (isLocalhost()) return false

    try {
      // Get the directory from the path
      const lastSlash = path.lastIndexOf('/')
      const directory = lastSlash > 0 ? path.substring(0, lastSlash) : ''

      // Load files from the index
      const files = await this.filesLoader.loadDirectory(directory)
      return files.some((f) => f.path === path)
    } catch {
      // Assume file doesn't exist on error
      return false
    }
  }

  private removeFile(id: string) {
    this.pendingFiles = this.pendingFiles.filter((f) => f.id !== id)
    this.savePendingFiles()
  }

  // Repo file browser (for update mode)

  private async fetchRepoFiles() {
    this.repoFilesLoading = true
    try {
      // Get current directory from document path
      const directory = this.effectiveTargetPath

      // Load files from the static _files index (available in local previews too)
      const entries = await this.filesLoader.loadDirectory(directory)

      // Filter by allowed extensions if configured
      const filtered = this.effectiveAllowedExtensions
        ? entries.filter((f) =>
            this.effectiveAllowedExtensions!.includes(f.extension.toLowerCase())
          )
        : entries

      // Map to RepoFile format
      this.repoFiles = filtered.map((f) => ({
        path: f.path,
        size: f.size,
        lastModified: f.lastModified,
      }))
    } catch {
      // Failed to load files
      this.repoFiles = []
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

    await this.processUpdateFile(files[0])
  }

  private async handleUpdateFileSelect(e: Event) {
    const input = e.target as HTMLInputElement
    const files = input.files
    if (!files || files.length === 0) return

    await this.processUpdateFile(files[0])
    input.value = '' // Reset input
  }

  private async processUpdateFile(file: File) {
    if (!this.selectedRepoFile) return

    // Validate size
    if (file.size > this.effectiveMaxFileSize) {
      this.errorMessage = `File too large: ${formatFileSize(file.size)} (max ${formatFileSize(this.effectiveMaxFileSize)})`
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
      // Clear previous PR success state when adding new files
      this.submittedPr = null
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

    this.isSubmitting = true
    this.submittedPr = null

    if (isLocalhost()) {
      // Show the payload that would be submitted (truncate content for display)
      this.showPreviewMock({
        endpoint: UPLOAD_SUBMIT_PATH,
        method: 'POST',
        body: {
          files: this.pendingFiles.map((f) => ({
            path: f.targetPath,
            content: f.content.length > 200
              ? f.content.substring(0, 200) + '... (truncated)'
              : f.content,
            size: f.size,
            mimeType: f.mimeType,
          })),
          message: this.message || undefined,
        },
      })
      this.isSubmitting = false
      this.clearPendingFiles()
      return
    }

    try {
      const response = await this.apiFetch(UPLOAD_SUBMIT_PATH, {
        method: 'POST',
        body: {
          files: this.pendingFiles.map((f) => ({
            path: f.targetPath,
            content: f.content,
            size: f.size,
            mimeType: f.mimeType,
          })),
          message: this.message || undefined,
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
   * Render upload-specific panel content (abstract method implementation)
   */
  protected override renderPanelContent() {
    return html`
      <!-- Tabs -->
      <div class="tabs" role="tablist" aria-label="Upload options">
        <button
          class="tab ${this.activeTab === 'upload' ? 'active' : ''}"
          role="tab"
          id="upload-tab"
          aria-selected=${this.activeTab === 'upload'}
          aria-controls="upload-panel"
          @click=${() => (this.activeTab = 'upload')}
        >
          Upload New
        </button>
        <button
          class="tab ${this.activeTab === 'update' ? 'active' : ''}"
          role="tab"
          id="update-tab"
          aria-selected=${this.activeTab === 'update'}
          aria-controls="update-panel"
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
      <div
        class="content"
        role="tabpanel"
        id="${this.activeTab}-panel"
        aria-labelledby="${this.activeTab}-tab"
      >
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
        role="region"
        aria-label="File upload drop zone"
        @dragover=${(e: DragEvent) => {
          e.preventDefault()
          this.isDragOver = true
        }}
        @dragleave=${() => (this.isDragOver = false)}
        @drop=${this.handleFileDrop}
      >
        <div class="dropzone-content">
          <p>Drop files here or click to browse</p>
          ${this.effectiveAllowedExtensions
            ? html`<p class="hint">
                Allowed: ${this.effectiveAllowedExtensions.join(', ')}
              </p>`
            : nothing}
          <input
            type="file"
            multiple
            @change=${this.handleFileSelect}
            aria-label="Select files to upload"
            accept=${this.effectiveAllowedExtensions
              ? this.effectiveAllowedExtensions.map((t) => `.${t}`).join(',')
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
      return html`<div class="empty">No files found in this directory</div>`
    }

    // Filter files by search query
    const filteredFiles = this.fileSearchQuery
      ? this.repoFiles.filter((file) => {
          const filename = file.path.split('/').pop() || file.path
          return filename.toLowerCase().includes(this.fileSearchQuery.toLowerCase())
        })
      : this.repoFiles

    return html`
      <div class="file-browser">
        <!-- Search filter for file lists with many items -->
        ${this.repoFiles.length > 5
          ? html`
              <div class="file-search">
                <span class="i-lucide:search file-search-icon"></span>
                <input
                  type="text"
                  class="file-search-input"
                  placeholder="Filter files..."
                  .value=${this.fileSearchQuery}
                  @input=${(e: Event) =>
                    (this.fileSearchQuery = (e.target as HTMLInputElement).value)}
                  aria-label="Filter files"
                />
              </div>
            `
          : nothing}

        <div class="file-list" role="listbox" aria-label="Files in current directory">
          ${filteredFiles.map((file) => {
            const filename = file.path.split('/').pop() || file.path
            const isSelected = this.selectedRepoFile?.path === file.path
            // Check if this file has a pending replacement
            const hasPendingReplacement = this.pendingFiles.some(
              (pf) => pf.targetPath === file.path && pf.isOverwrite
            )

            return html`
              <div
                class="file-item ${isSelected ? 'selected' : ''} ${hasPendingReplacement ? 'has-pending' : ''}"
                role="option"
                aria-selected=${isSelected}
                tabindex="0"
                @click=${() => this.selectRepoFile(file)}
                @keydown=${(e: KeyboardEvent) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault()
                    this.selectRepoFile(file)
                  }
                }}
              >
                <span class="file-icon">
                  <span class="i-lucide:${getFileIcon(filename)}"></span>
                </span>
                <span class="file-name">${filename}</span>
                <span class="file-size">${formatFileSize(file.size)}</span>
                ${hasPendingReplacement
                  ? html`
                      <span class="pending-indicator" title="Replacement queued">
                        <span class="i-lucide:clock"></span>
                      </span>
                    `
                  : nothing}
              </div>
            `
          })}
        </div>

        <div
          class="dropzone update-dropzone ${this.isDragOver ? 'drag-over' : ''} ${!this.selectedRepoFile ? 'disabled' : ''}"
          @dragover=${(e: DragEvent) => {
            if (!this.selectedRepoFile) return
            e.preventDefault()
            this.isDragOver = true
          }}
          @dragleave=${() => (this.isDragOver = false)}
          @drop=${this.handleUpdateFile}
        >
          <span class="i-lucide:replace dropzone-icon" aria-hidden="true"></span>
          ${this.selectedRepoFile
            ? html`<p>Drop a file, or click to select, a file to replace <strong>${this.selectedRepoFile.path.split('/').pop()}</strong></p>`
            : html`<p class="hint">Select a file from the list above first</p>`}
          <input
            type="file"
            @change=${this.handleUpdateFileSelect}
            aria-label="Select replacement file"
            ?disabled=${!this.selectedRepoFile}
          />
        </div>
      </div>
    `
  }

  private renderPendingFiles() {
    if (this.pendingFiles.length === 0) return nothing

    return html`
      <div class="pending">
        <ul class="pending-list">
          ${this.pendingFiles.map(
            (file) => html`
              <li class="pending-file">
                <div class="file-info">
                  <span class="file-icon"><span class="i-lucide:${getFileIcon(file.filename)}"></span></span>
                  <span class="file-name">${file.filename}</span>
                  ${file.isOverwrite
                    ? html`<span class="file-badge update">update</span>`
                    : html`<span class="file-badge new">new</span>`}
                </div>
                <div class="file-path">
                  <span>${file.targetPath}</span>
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
        <textarea
          class="message-input"
          placeholder="Briefly describe what you're uploading and why..."
          aria-label="Pull request description"
          .value=${this.message}
          @input=${(e: Event) =>
            (this.message = (e.target as HTMLTextAreaElement).value)}
        ></textarea>
        <p class="message-hint">This will be the commit message for your pull request</p>
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
