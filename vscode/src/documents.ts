import * as path from 'path'

import * as vscode from 'vscode'
import { LanguageClient } from 'vscode-languageclient/node'

import { event } from './events'

export function registerDocumentsView(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const treeDataProvider = new DocumentsTreeProvider(client, context)

  const treeView = vscode.window.createTreeView('stencila-documents', {
    treeDataProvider,
    showCollapseAll: true,
  })

  // Refresh on file save
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((document) => {
      const fileName = path.basename(document.fileName)
      // Check if it's a Stencila supported document
      if (
        fileName.endsWith('.smd') ||
        fileName.endsWith('.md') ||
        fileName.endsWith('.qmd') ||
        fileName.endsWith('.myst')
      ) {
        treeDataProvider.refresh()
      }
    })
  )

  // List command
  const list = vscode.commands.registerCommand(
    'stencila.documents.list',
    async () => {
      return await treeDataProvider.getDocumentTracking()
    }
  )

  // Refresh command
  const refresh = vscode.commands.registerCommand(
    'stencila.documents.refresh',
    () => {
      event('documents_refresh')
      treeDataProvider.refresh()
    }
  )

  // Refresh specific document
  const refreshDocument = vscode.commands.registerCommand(
    'stencila.documents.refreshDocument',
    (_item: DocumentTreeItem) => {
      event('documents_refresh_document')
      treeDataProvider.refresh()
    }
  )

  // Push to all remotes
  const pushAllRemotes = vscode.commands.registerCommand(
    'stencila.documents.pushAllRemotes',
    async (item: DocumentTreeItem) => {
      event('documents_push_all_remotes')
      await pushDocumentToAllRemotes(item.filePath)
      treeDataProvider.refresh()
    }
  )

  // Pull from remote (with picker if multiple remotes)
  const pullFromRemote = vscode.commands.registerCommand(
    'stencila.documents.pullFromRemote',
    async (item: DocumentTreeItem) => {
      event('documents_pull_from_remote')
      await pullDocumentFromRemote(item.filePath, item.tracking)
      treeDataProvider.refresh()
    }
  )

  // Watch all remotes
  const watchAllRemotes = vscode.commands.registerCommand(
    'stencila.documents.watchAllRemotes',
    async (item: DocumentTreeItem) => {
      event('documents_watch_all_remotes')
      await watchAllDocumentRemotes(item.filePath, item.tracking)
      treeDataProvider.refresh()
    }
  )

  // Unwatch all remotes
  const unwatchAllRemotes = vscode.commands.registerCommand(
    'stencila.documents.unwatchAllRemotes',
    async (item: DocumentTreeItem) => {
      event('documents_unwatch_all_remotes')
      await unwatchAllDocumentRemotes(item.filePath, item.tracking)
      treeDataProvider.refresh()
    }
  )

  // Push to new remote
  const pushToNewRemote = vscode.commands.registerCommand(
    'stencila.documents.pushToNewRemote',
    async (item: DocumentTreeItem) => {
      event('documents_push_to_new_remote')
      await pushToNewRemote_(item.filePath)
      treeDataProvider.refresh()
    }
  )

  // Push remote
  const pushRemote = vscode.commands.registerCommand(
    'stencila.documents.pushRemote',
    async (item: RemoteTreeItem) => {
      event('documents_push_remote')
      await pushDocumentToRemote(item.documentPath, item.remoteUrl)
      treeDataProvider.refresh()
    }
  )

  // Pull remote
  const pullRemote = vscode.commands.registerCommand(
    'stencila.documents.pullRemote',
    async (item: RemoteTreeItem) => {
      event('documents_pull_remote')
      await pullDocumentFromRemoteUrl(item.documentPath, item.remoteUrl)
      treeDataProvider.refresh()
    }
  )

  // Start watch
  const startWatch = vscode.commands.registerCommand(
    'stencila.documents.startWatch',
    async (item: RemoteTreeItem) => {
      event('documents_start_watch')
      await startWatchRemote(item.documentPath, item.remoteUrl)
      treeDataProvider.refresh()
    }
  )

  // Stop watch
  const stopWatch = vscode.commands.registerCommand(
    'stencila.documents.stopWatch',
    async (item: RemoteTreeItem) => {
      event('documents_stop_watch')
      await stopWatchRemote(item.documentPath, item.remoteUrl)
      treeDataProvider.refresh()
    }
  )

  // Copy remote URL
  const copyUrl = vscode.commands.registerCommand(
    'stencila.documents.copyUrl',
    async (item: RemoteTreeItem) => {
      event('documents_copy_url')
      await vscode.env.clipboard.writeText(item.remoteUrl)
      vscode.window.showInformationMessage('URL copied to clipboard')
    }
  )

  // Open remote
  const openRemote = vscode.commands.registerCommand(
    'stencila.documents.openRemote',
    async (item: RemoteTreeItem) => {
      event('documents_open_remote')
      await vscode.env.openExternal(vscode.Uri.parse(item.remoteUrl))
    }
  )

  // Open PR
  const openPr = vscode.commands.registerCommand(
    'stencila.documents.openPr',
    async (item: RemoteTreeItem) => {
      event('documents_open_pr')
      if (item.remote.currentPr) {
        await vscode.env.openExternal(vscode.Uri.parse(item.remote.currentPr.url))
      }
    }
  )

  // Open document
  const openDocument = vscode.commands.registerCommand(
    'stencila.documents.openDocument',
    async (item: DocumentTreeItem) => {
      event('documents_open_document')
      try {
        // Convert path to URI - handle both absolute and relative paths
        let fileUri: vscode.Uri
        if (path.isAbsolute(item.filePath)) {
          fileUri = vscode.Uri.file(item.filePath)
        } else {
          // Resolve relative path against workspace folder
          const workspaceFolder = vscode.workspace.workspaceFolders?.[0]
          if (!workspaceFolder) {
            throw new Error('No workspace folder open')
          }
          const absolutePath = path.join(workspaceFolder.uri.fsPath, item.filePath)
          fileUri = vscode.Uri.file(absolutePath)
        }

        const document = await vscode.workspace.openTextDocument(fileUri)
        await vscode.window.showTextDocument(document)
      } catch (error) {
        vscode.window.showErrorMessage(`Failed to open document: ${error}`)
      }
    }
  )

  // Workspace-level: Push all documents
  const pushAllDocuments = vscode.commands.registerCommand(
    'stencila.documents.pushAllDocuments',
    async () => {
      event('documents_push_all_documents')
      const tracking = await treeDataProvider.getDocumentTracking()
      for (const [filePath] of Object.entries(tracking)) {
        try {
          await pushDocumentToAllRemotes(filePath)
        } catch (error) {
          vscode.window.showErrorMessage(
            `Failed to push ${path.basename(filePath)}: ${error}`
          )
        }
      }
      treeDataProvider.refresh()
    }
  )

  // Workspace-level: Sync all watches
  const syncAllWatches = vscode.commands.registerCommand(
    'stencila.documents.syncAllWatches',
    async () => {
      event('documents_sync_all_watches')
      vscode.window.showInformationMessage(
        'Syncing all watches... This may take a moment.'
      )
      // Note: The actual syncing happens on the server side via watches
      // This just triggers a refresh to show updated status
      setTimeout(() => {
        treeDataProvider.refresh()
      }, 2000)
    }
  )

  context.subscriptions.push(
    treeView,
    list,
    refresh,
    refreshDocument,
    pushAllRemotes,
    pullFromRemote,
    watchAllRemotes,
    unwatchAllRemotes,
    pushToNewRemote,
    pushRemote,
    pullRemote,
    startWatch,
    stopWatch,
    copyUrl,
    openRemote,
    openPr,
    openDocument,
    pushAllDocuments,
    syncAllWatches
  )

  return treeDataProvider
}

/**
 * Data structures matching Rust types
 */

interface DocumentTracking {
  id: string
  cachedAt?: number
  addedAt?: number
  remotes?: Record<string, DocumentRemote>
}

interface DocumentRemote {
  pulledAt?: number
  pushedAt?: number
  watchId?: string
  watchDirection?: 'bi' | 'from-remote' | 'to-remote'
  serviceName?: string
  displayName?: string
  status?: DocumentTrackingStatus
  watchStatus?: 'Ok' | 'Pending' | 'Syncing' | 'Blocked' | 'Error'
  watchStatusSummary?: string
  watchLastError?: string
  currentPr?: {
    status: string
    url: string
  }
}

type DocumentTrackingEntries = Record<string, DocumentTracking>

enum DocumentTrackingStatus {
  Unknown = 'Unknown',
  Deleted = 'Deleted',
  Ahead = 'Ahead',
  Behind = 'Behind',
  Diverged = 'Diverged',
  Synced = 'Synced',
  Error = 'Error',
}

/**
 * Tree item types
 */

type TreeItemType = DocumentTreeItem | RemoteTreeItem | DetailTreeItem

class DocumentTreeItem extends vscode.TreeItem {
  constructor(
    public readonly filePath: string,
    public readonly tracking: DocumentTracking,
    public readonly status: DocumentTrackingStatus,
    context: vscode.ExtensionContext
  ) {
    const fileName = path.basename(filePath)
    super(fileName, vscode.TreeItemCollapsibleState.Expanded)

    this.id = filePath
    this.description = getStatusLabel(status)
    this.tooltip = getDocumentTooltip(filePath, tracking, status)
    this.iconPath = getDocumentIcon(filePath, context)
    this.contextValue = 'document'
    this.resourceUri = vscode.Uri.file(filePath)

    // Allow opening the document on click
    this.command = {
      command: 'stencila.documents.openDocument',
      title: 'Open Document',
      arguments: [this],
    }
  }
}

class RemoteTreeItem extends vscode.TreeItem {
  constructor(
    public readonly documentPath: string,
    public readonly remoteUrl: string,
    public readonly remote: DocumentRemote,
    public readonly status: DocumentTrackingStatus,
    context: vscode.ExtensionContext
  ) {
    const statusLabel = getStatusLabel(status)

    // Use display name if available, otherwise show URL
    const label = remote.displayName || remoteUrl

    // Only show expand caret if there are details to show
    const hasDetails = remote.watchId || remote.pushedAt || remote.pulledAt
    const collapsibleState = hasDetails
      ? vscode.TreeItemCollapsibleState.Collapsed
      : vscode.TreeItemCollapsibleState.None

    super(label, collapsibleState)

    this.id = `${documentPath}:${remoteUrl}`
    this.description = `${statusLabel}`
    this.tooltip = getRemoteTooltip(documentPath, remoteUrl, remote, status)
    this.iconPath = getRemoteIcon(remote, status, context)

    // Set context value based on watch and PR status
    if (remote.watchId && remote.currentPr) {
      this.contextValue = 'remote-watching-pr'
    } else if (remote.watchId) {
      this.contextValue = 'remote-watching'
    } else {
      this.contextValue = 'remote'
    }

    // Allow opening the remote in browser on click
    this.command = {
      command: 'stencila.documents.openRemote',
      title: 'Open Remote in Browser',
      arguments: [this],
    }
  }
}

class DetailTreeItem extends vscode.TreeItem {
  constructor(
    label: string,
    description?: string,
    tooltip?: string,
    iconId?: string
  ) {
    super(label, vscode.TreeItemCollapsibleState.None)

    this.description = description
    this.tooltip = tooltip
    this.iconPath = new vscode.ThemeIcon(iconId || 'info')
    this.contextValue = 'detail'
  }
}

/**
 * Tree data provider
 */

class DocumentsTreeProvider
  implements vscode.TreeDataProvider<TreeItemType> {
  client: LanguageClient
  context: vscode.ExtensionContext
  private trackingCache?: DocumentTrackingEntries

  private _onDidChangeTreeData: vscode.EventEmitter<
    TreeItemType | undefined | null | void
  > = new vscode.EventEmitter<TreeItemType | undefined | null | void>()
  readonly onDidChangeTreeData: vscode.Event<
    TreeItemType | undefined | null | void
  > = this._onDidChangeTreeData.event

  constructor(client: LanguageClient, context: vscode.ExtensionContext) {
    this.client = client
    this.context = context
  }

  refresh(client?: LanguageClient): void {
    if (client) {
      this.client = client
    }

    this.trackingCache = undefined
    this._onDidChangeTreeData.fire()
  }

  getTreeItem(item: TreeItemType): vscode.TreeItem {
    return item
  }

  async getChildren(item?: TreeItemType): Promise<TreeItemType[]> {
    if (!item) {
      // Root level: return documents
      const tracking = await this.getDocumentTracking()
      const documents: DocumentTreeItem[] = []

      // Get workspace folder to resolve relative paths
      const workspaceFolder = vscode.workspace.workspaceFolders?.[0]

      for (const [filePath, docTracking] of Object.entries(tracking)) {
        // Only show documents that have remotes
        if (docTracking.remotes && Object.keys(docTracking.remotes).length > 0) {
          // Resolve relative paths to absolute paths
          const absolutePath = workspaceFolder && !path.isAbsolute(filePath)
            ? path.join(workspaceFolder.uri.fsPath, filePath)
            : filePath

          const status = calculateDocumentStatus(docTracking)
          documents.push(new DocumentTreeItem(absolutePath, docTracking, status, this.context))
        }
      }

      return documents
    }

    if (item instanceof DocumentTreeItem) {
      // Second level: return remotes for this document
      const remotes: TreeItemType[] = []

      if (item.tracking.remotes) {
        for (const [url, remote] of Object.entries(item.tracking.remotes)) {
          const status = calculateRemoteStatus(remote)
          remotes.push(new RemoteTreeItem(item.filePath, url, remote, status, this.context))
        }
      }

      return remotes
    }

    if (item instanceof RemoteTreeItem) {
      // Third level: return details for this remote
      return getRemoteDetails(item.remote)
    }

    return []
  }

  async getDocumentTracking(): Promise<DocumentTrackingEntries> {
    if (this.trackingCache) {
      return this.trackingCache
    }

    try {
      // Use LSP command to get tracking status
      const result = await this.client.sendRequest<DocumentTrackingEntries>(
        'stencila.documents/tracking'
      )
      this.trackingCache = result
      return result
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to fetch document tracking: ${error}`
      )
      return {}
    }
  }
}

/**
 * Helper functions
 */

function getStatusLabel(status: DocumentTrackingStatus): string {
  switch (status) {
    case DocumentTrackingStatus.Synced:
    case DocumentTrackingStatus.Behind:
    case DocumentTrackingStatus.Ahead:
    case DocumentTrackingStatus.Diverged:
    case DocumentTrackingStatus.Error:
    case DocumentTrackingStatus.Deleted:
      return status.toString()
    default:
      return 'Unknown'
  }
}

function calculateDocumentStatus(
  tracking: DocumentTracking
): DocumentTrackingStatus {
  if (!tracking.remotes || Object.keys(tracking.remotes).length === 0) {
    return DocumentTrackingStatus.Unknown
  }

  // Return the most critical status across all remotes
  // Priority: Error > Diverged > Ahead > Behind > Synced
  let mostCritical = DocumentTrackingStatus.Synced

  for (const remote of Object.values(tracking.remotes)) {
    const status = calculateRemoteStatus(remote)

    // Early return for Error status (highest priority)
    if (status === DocumentTrackingStatus.Error) {
      return DocumentTrackingStatus.Error
    }

    // Update mostCritical based on priority: Diverged > Ahead > Behind > Synced
    if (status === DocumentTrackingStatus.Diverged) {
      mostCritical = DocumentTrackingStatus.Diverged
    } else if (
      status === DocumentTrackingStatus.Ahead &&
      mostCritical !== DocumentTrackingStatus.Diverged
    ) {
      mostCritical = DocumentTrackingStatus.Ahead
    } else if (
      status === DocumentTrackingStatus.Behind &&
      mostCritical === DocumentTrackingStatus.Synced
    ) {
      mostCritical = DocumentTrackingStatus.Behind
    }
  }

  return mostCritical
}

function calculateRemoteStatus(
  remote: DocumentRemote
): DocumentTrackingStatus {
  // Use backend-provided status if available (calculated by comparing with remote)
  if (remote.status) {
    return remote.status
  }

  // Fallback to Unknown if backend hasn't calculated status yet
  return DocumentTrackingStatus.Unknown
}

function getDocumentIcon(
  filePath: string,
  context: vscode.ExtensionContext
): vscode.Uri | vscode.ThemeIcon {
  const ext = path.extname(filePath)
  switch (ext) {
    case '.smd':
      return vscode.Uri.joinPath(context.extensionUri, 'icons/stencila-128.png')
    case '.md':
      return new vscode.ThemeIcon('markdown')
    case '.qmd':
      return vscode.Uri.joinPath(context.extensionUri, 'icons/quarto-128.png')
    case '.myst':
      return vscode.Uri.joinPath(context.extensionUri, 'icons/myst-128.png')
    case '.tex':
    case '.latex':
      return new vscode.ThemeIcon('symbol-misc')
    case '.ipynb':
      return new vscode.ThemeIcon('notebook')
    default:
      return new vscode.ThemeIcon('file')
  }
}

function getRemoteIcon(
  remote: DocumentRemote,
  status: DocumentTrackingStatus,
  context: vscode.ExtensionContext
): vscode.Uri | vscode.ThemeIcon {
  if (status === DocumentTrackingStatus.Error) {
    return new vscode.ThemeIcon('error')
  }

  // Use service-specific icons if we have service information
  if (remote.serviceName) {
    switch (remote.serviceName) {
      case 'gdocs':
        return vscode.Uri.joinPath(
          context.extensionUri,
          'icons/google-docs.png'
        )
      case 'm365':
        return vscode.Uri.joinPath(
          context.extensionUri,
          'icons/microsoft-365.png'
        )
      default:
        return new vscode.ThemeIcon('symbol-namespace')
    }
  }

  // Fallback to generic icons
  return new vscode.ThemeIcon('link')
}

function getDocumentTooltip(
  filePath: string,
  tracking: DocumentTracking,
  status: DocumentTrackingStatus
): vscode.MarkdownString {
  const tooltip = new vscode.MarkdownString()
  tooltip.isTrusted = true

  tooltip.appendMarkdown(`#### ${path.basename(filePath)}\n\n`)

  tooltip.appendMarkdown(`**Status**: ${getStatusLabel(status)}. `)

  if (tracking.remotes) {
    const remoteCount = Object.keys(tracking.remotes).length
    tooltip.appendMarkdown(`**Remotes**: ${remoteCount}. `)

    const watchingCount = Object.values(tracking.remotes).filter(
      (r) => r.watchId
    ).length
    if (watchingCount > 0) {
      tooltip.appendMarkdown(`**Watching**: ${watchingCount} remote(s).`)
    }
  }

  return tooltip
}

function getRemoteTooltip(
  documentPath: string,
  url: string,
  remote: DocumentRemote,
  status: DocumentTrackingStatus
): vscode.MarkdownString {
  const tooltip = new vscode.MarkdownString()
  tooltip.isTrusted = true

  const fileName = path.basename(documentPath)
  const remoteName = remote.displayName || remote.serviceName || 'Remote'

  tooltip.appendMarkdown(`#### ${fileName} to/from [${remoteName}](${url})\n\n`)

  tooltip.appendMarkdown(`**Status**: ${getStatusLabel(status)}. `)

  if (remote.pushedAt) {
    tooltip.appendMarkdown(
      `**Last pushed**: ${humanizeTimestamp(remote.pushedAt)}. `
    )
  }

  if (remote.pulledAt) {
    tooltip.appendMarkdown(
      `**Last pulled**: ${humanizeTimestamp(remote.pulledAt)}.\n\n`
    )
  }

  if (remote.watchId) {
    tooltip.appendMarkdown(
      `**Watch**: ${getWatchDirectionName(remote.watchDirection)}`
    )

    if (remote.watchStatus) {
      tooltip.appendMarkdown(` - ${remote.watchStatus}`)
    }

    tooltip.appendMarkdown('. ')
  }

  if (remote.currentPr) {
    tooltip.appendMarkdown(
      `**Pull Request**: [${formatPRStatus(remote.currentPr.status)}](${remote.currentPr.url})`
    )
  }

  return tooltip
}

function getRemoteDetails(remote: DocumentRemote): DetailTreeItem[] {
  const details: DetailTreeItem[] = []

  if (remote.pushedAt) {
    details.push(
      new DetailTreeItem(
        'Last pushed',
        humanizeTimestamp(remote.pushedAt),
        isoizeTimestamp(remote.pushedAt),
        'cloud-upload'
      )
    )
  }

  if (remote.pulledAt) {
    details.push(
      new DetailTreeItem(
        'Last pulled',
        humanizeTimestamp(remote.pulledAt),
        isoizeTimestamp(remote.pulledAt),
        'cloud-download'
      )
    )
  }

  if (remote.watchId) {
    details.push(
      new DetailTreeItem(
        'Watch',
        getWatchDirectionName(remote.watchDirection),
        `#${remote.watchId}`,
        'eye'
      )
    )
  }

  if (remote.currentPr) {
    details.push(
      new DetailTreeItem(
        'Pull Request',
        formatPRStatus(remote.currentPr.status),
        remote.currentPr.url,
        'git-pull-request'
      )
    )
  }

  return details
}

function getWatchDirectionName(
  direction?: 'bi' | 'from-remote' | 'to-remote'
): string {
  switch (direction?.toLowerCase()) {
    case 'from-remote':
      return 'From remote'
    case 'to-remote':
      return 'To remote'
    case 'bi':
    default:
      return 'Bi-directional'
  }
}

function formatPRStatus(status: string): string {
  return status.charAt(0).toUpperCase() + status.slice(1).toLowerCase()
}

function humanizeTimestamp(timestamp: number): string {
  const now = Date.now() / 1000
  const diff = now - timestamp

  if (diff < 60) {
    return 'just now'
  } else if (diff < 3600) {
    const mins = Math.floor(diff / 60)
    return `${mins}m ago`
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600)
    return `${hours}h ago`
  } else {
    const days = Math.floor(diff / 86400)
    return `${days}d ago`
  }
}

function isoizeTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toISOString().replace(/\.000Z$/, 'Z')
}

/**
 * Command implementations
 */

async function pushDocumentToAllRemotes(filePath: string): Promise<void> {
  try {
    await vscode.commands.executeCommand('stencila.push-doc', filePath, {
      push_all_remotes: true,
      force_new: false,
      no_execute: false,
      watch: false,
      args: '',
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to push document: ${error}`)
  }
}

async function pushDocumentToRemote(
  filePath: string,
  remoteUrl: string
): Promise<void> {
  try {
    await vscode.commands.executeCommand('stencila.push-doc', filePath, {
      target: remoteUrl,
      push_all_remotes: false,
      force_new: false,
      no_execute: false,
      watch: false,
      args: '',
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to push to remote: ${error}`)
  }
}

async function pullDocumentFromRemote(
  filePath: string,
  tracking: DocumentTracking
): Promise<void> {
  if (!tracking.remotes || Object.keys(tracking.remotes).length === 0) {
    vscode.window.showInformationMessage('No remotes to pull from')
    return
  }

  const remoteUrls = Object.keys(tracking.remotes)

  let target: string | undefined

  if (remoteUrls.length === 1) {
    target = remoteUrls[0]
  } else {
    const selected = await vscode.window.showQuickPick(
      remoteUrls.map((url) => ({
        label: url,
        value: url,
      })),
      {
        title: 'Select remote to pull from',
        placeHolder: 'Choose a remote',
      }
    )
    if (!selected) {
      return
    }
    target = selected.value
  }

  try {
    await vscode.commands.executeCommand('stencila.pull-doc', filePath, {
      target,
      merge: false,
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to pull from remote: ${error}`)
  }
}

async function pullDocumentFromRemoteUrl(
  filePath: string,
  remoteUrl: string
): Promise<void> {
  try {
    await vscode.commands.executeCommand('stencila.pull-doc', filePath, {
      target: remoteUrl,
      merge: false,
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to pull from remote: ${error}`)
  }
}

async function watchAllDocumentRemotes(
  filePath: string,
  tracking: DocumentTracking
): Promise<void> {
  if (!tracking.remotes) {
    vscode.window.showInformationMessage('No remotes to watch')
    return
  }

  const direction = await vscode.window.showQuickPick(
    [
      { label: 'Bi-directional', value: 'bi' },
      { label: 'From remote', value: 'from-remote' },
      { label: 'To remote', value: 'to-remote' },
    ],
    {
      title: 'Select watch direction',
      placeHolder: 'Choose sync direction',
    }
  )

  if (!direction) {
    return
  }

  for (const url of Object.keys(tracking.remotes)) {
    try {
      await vscode.commands.executeCommand('stencila.watch-doc', filePath, {
        target: url,
        direction: direction.value,
      })
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to watch ${url}: ${error}`
      )
    }
  }
}

async function unwatchAllDocumentRemotes(
  filePath: string,
  tracking: DocumentTracking
): Promise<void> {
  if (!tracking.remotes) {
    return
  }

  for (const url of Object.keys(tracking.remotes)) {
    try {
      await vscode.commands.executeCommand('stencila.unwatch-doc', filePath, {
        target: url,
      })
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to unwatch ${url}: ${error}`
      )
    }
  }
}

async function pushToNewRemote_(filePath: string): Promise<void> {
  const serviceChoice = await vscode.window.showQuickPick(
    [
      {
        label: 'Google Docs',
        value: 'gdoc',
        description: 'Create a new Google Doc',
      },
      {
        label: 'Microsoft 365',
        value: 'm365',
        description: 'Create a new Microsoft 365 document',
      },
      {
        label: 'Enter URL manually',
        value: 'manual',
        description: 'Connect to or update an existing remote',
      },
    ],
    {
      title: 'Select service to push to',
      placeHolder: 'Choose where to push this document.',
    }
  )

  if (!serviceChoice) {
    return
  }

  let target = serviceChoice.value
  let forceNew = true

  if (serviceChoice.value === 'manual') {
    const urlInput = await vscode.window.showInputBox({
      title: 'Enter remote URL',
      placeHolder: 'https://docs.google.com/document/d/...',
      prompt: 'Enter the full URL of the remote document',
    })
    if (!urlInput) {
      return
    }
    target = urlInput
    forceNew = false
  }

  try {
    await vscode.commands.executeCommand('stencila.push-doc', filePath, {
      target,
      force_new: forceNew,
      push_all_remotes: false,
      no_execute: false,
      watch: false,
      args: '',
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to push to new remote: ${error}`)
  }
}

async function startWatchRemote(
  filePath: string,
  remoteUrl: string
): Promise<void> {
  const direction = await vscode.window.showQuickPick(
    [
      { label: 'Bi-directional', value: 'bi' },
      { label: 'From remote', value: 'from-remote' },
      { label: 'To remote', value: 'to-remote' },
    ],
    {
      title: 'Select watch direction',
      placeHolder: 'Choose sync direction',
    }
  )

  if (!direction) {
    return
  }

  try {
    await vscode.commands.executeCommand('stencila.watch-doc', filePath, {
      target: remoteUrl,
      direction: direction.value,
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to start watch: ${error}`)
  }
}

async function stopWatchRemote(
  filePath: string,
  remoteUrl: string
): Promise<void> {
  try {
    await vscode.commands.executeCommand('stencila.unwatch-doc', filePath, {
      target: remoteUrl,
    })
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to stop watch: ${error}`)
  }
}
