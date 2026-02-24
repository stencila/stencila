/* eslint-disable @typescript-eslint/no-unused-vars */

import * as vscode from 'vscode'

import { event } from './events'
import { createDocumentViewPanel, createNodeViewPanel } from './webviews'

class Format {
  constructor(
    public description: string,
    public format: string,
    public render = false,
    public reproducible = false
  ) {}
}

const FORMATS = {
  'repro-docx': new Format('Reproducible Microsoft Word', 'docx', true, true),
  _1: null,
  docx: new Format('Microsoft Word', 'docx'),
  odt: new Format('Open Document Text', 'odt'),
  pdf: new Format('Adobe Portable Document Format', 'pdf'),
  _2: null,
  tex: new Format('LaTeX', 'latex'),
  myst: new Format('MyST Markdown', 'myst'),
  qmd: new Format('Quarto Markdown', 'qmd'),
  smd: new Format('Stencila Markdown', 'smd'),
  _3: null,
  json: new Format('Stencila Schema JSON', 'json'),
  jsonld: new Format('Schema.org JSON Linked Data', 'jsonld'),
  yaml: new Format('Stencila Schema YAML', 'yaml'),
}

function formatQuickPickItems() {
  return Object.entries(FORMATS).map(([label, format]) =>
    format
      ? {
          label,
          description: format.description,
        }
      : {
          label: '',
          kind: vscode.QuickPickItemKind.Separator,
        }
  )
}

function getFormat(label: keyof typeof FORMATS) {
  return FORMATS[label]
}

/**
 * Register document related commands provided by the extension
 */
export function registerDocumentCommands(context: vscode.ExtensionContext) {
  // Keep track of the most recently active text editor as a fallback in
  // commands below
  let lastTextEditor: vscode.TextEditor | null = null
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (
        editor?.document.languageId &&
        ['smd', 'myst', 'qmd', 'latex'].includes(editor?.document.languageId)
      ) {
        lastTextEditor = editor
      }
    })
  )

  // Create document commands
  for (const format of ['smd', 'myst', 'qmd', 'latex']) {
    context.subscriptions.push(
      vscode.commands.registerCommand(`stencila.new-${format}`, async () => {
        event('doc_create', { format })

        vscode.workspace.openTextDocument({ language: format }).then(
          (document) => {
            vscode.window.showTextDocument(document)
          },
          (err) => {
            vscode.window.showErrorMessage(
              `Failed to create new '${format}' file: ${err.message}`
            )
          }
        )
      })
    )
  }

  // Create a new chat document and open with the chat editor
  vscode.commands.registerCommand(`stencila.new-chat`, async () => {
    event('chat_create')

    const doc = await vscode.workspace.openTextDocument({
      language: 'smd',
      content: `---
type: Chat
---
`,
    })

    await createDocumentViewPanel(
      context,
      doc.uri,
      undefined,
      undefined,
      false,
      vscode.ViewColumn.Active,
      'Chat'
    )
  })

  // Create a new prompt
  vscode.commands.registerCommand(`stencila.new-prompt`, async () => {
    // TODO: ask user for required fields, e.g instruction types, node types

    event('prompt_create')

    await vscode.workspace.openTextDocument({
      language: 'smd',
      content: `---
type: Prompt
name: user/type/name
version: 0.1.0
description: description
instructionTypes: []
nodeTypes: []
---
`,
    })
  })

  // Commands executed by the server but which are invoked on the client
  // and which use are passed the document URI and selection (position) as arguments
  for (const command of [
    'run-below',
    'run-above',
    'run-doc',
    'run-code',
    'run-instruct',
    'cancel-curr',
    'cancel-doc',
    'lock-curr',
    'unlock-curr',
    'prev-node',
    'next-node',
    'archive-node',
  ]) {
    context.subscriptions.push(
      vscode.commands.registerCommand(`stencila.invoke.${command}`, () => {
        const editor = vscode.window.activeTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        vscode.commands.executeCommand(
          `stencila.${command}`,
          editor.document.uri.toString(),
          editor.selection.active
        )
      })
    )
  }

  // Run the current node
  vscode.commands.registerCommand(`stencila.invoke.run-curr`, async () => {
    const editor = vscode.window.activeTextEditor
    if (!editor) {
      vscode.window.showErrorMessage('No active editor')
      return
    }

    const result = await vscode.commands.executeCommand(
      `stencila.run-curr`,
      editor.document.uri.toString(),
      editor.selection.active
    )

    let nodeType
    let nodeId
    if (
      Array.isArray(result) &&
      typeof result[0] === 'string' &&
      typeof result[1] === 'string'
    ) {
      nodeType = result[0]
      nodeId = result[1]
    } else {
      return
    }

    if (nodeType === 'Chat') {
      await createNodeViewPanel(
        context,
        editor.document.uri,
        null,
        nodeType,
        nodeId
      )
    }
  })

  // Retry the active suggestion without feedback
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.invoke.retry-node',
      async (_docUri, _nodeType, _nodeId) => {
        const editor = vscode.window.activeTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        vscode.commands.executeCommand(
          `stencila.revise-node`,
          editor.document.uri.toString(),
          editor.selection.active
        )
      }
    )
  )

  // Revise the active suggestion of an instruction with feedback
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.invoke.revise-node',
      async (_docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        const feedback = await vscode.window.showInputBox({
          title: 'Revise suggestion',
          placeHolder:
            'Describe what you want changed, or leave blank to just retry.',
        })

        vscode.commands.executeCommand(
          `stencila.revise-node`,
          editor.document.uri.toString(),
          // If invoked from code lens then `nodeType` and `nodeId` should be defined
          // and should be passed as arguments. Otherwise, if invoked using keybinding
          // then those arguments will not be present so pass the selection.
          ...(nodeId ? [nodeType, nodeId] : [editor.selection.active]),
          feedback
        )
      }
    )
  )

  // Insert a clone of a node
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.invoke.insert-clones',
      async (docUri, [nodeIds]) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        vscode.commands.executeCommand(
          `stencila.insert-clones`,
          // For consistency, first args are destination document and position
          editor.document.uri.toString(),
          editor.selection.active,
          // Source document and nodes
          docUri,
          nodeIds
        )
      }
    )
  )

  // Insert a clone of a node with an instruction to edit, fix or update it
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.invoke.insert-instruction',
      async (docUri, nodeType, nodeId, instructionType, executionMode) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        vscode.commands.executeCommand(
          `stencila.insert-instruction`,
          // For consistency, first args are destination document and position
          editor.document.uri.toString(),
          editor.selection.active,
          // Source document and node
          docUri,
          nodeType,
          nodeId,
          // Instruction properties
          instructionType,
          executionMode
        )
      }
    )
  )

  // Export document command which requires user entered file path
  // so must be invoked from here
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.export-doc', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      const item = await vscode.window.showQuickPick(formatQuickPickItems(), {
        title: 'Export Format',
        placeHolder: 'Select a format to export the document to',
        matchOnDescription: true,
      })

      if (!item) {
        vscode.window.showInformationMessage('Document conversion cancelled.')
        return
      }

      const options = getFormat(item.label as keyof typeof FORMATS)!

      const filename = editor.document.fileName
      const saveUri = await vscode.window.showSaveDialog({
        title: 'Export Document',
        saveLabel: 'Export',
        defaultUri: vscode.Uri.file(
          `${filename.substring(0, filename.lastIndexOf('.'))}.${options.format}`
        ),
      })

      if (!saveUri) {
        vscode.window.showInformationMessage('Document export cancelled.')
        return
      }

      event('doc_export', options)

      vscode.commands.executeCommand(
        `stencila.export-doc`,
        editor.document.uri.toString(),
        saveUri.fsPath,
        options.format,
        options.render,
        options.reproducible
      )
    })
  )

  // Merge document command which requires user entered file path
  // so must be invoked from here
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.merge-doc', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      // Save the document if it has unsaved changes
      if (editor.document.isDirty) {
        const saved = await editor.document.save()
        if (!saved) {
          vscode.window.showInformationMessage(
            'Document merge cancelled: document must be saved first.'
          )
          return
        }
      }

      const original = editor.document.uri.fsPath

      const fileUri = await vscode.window.showOpenDialog({
        canSelectFiles: true,
        canSelectFolders: false,
        canSelectMany: false,
        openLabel: 'Merge',
        title: 'Select file to merge into document',
      })

      if (!fileUri || fileUri.length === 0) {
        return
      }

      const edited = fileUri[0].fsPath

      event('doc_merge')

      const filesModified: string[] = await vscode.commands.executeCommand(
        `stencila.merge-doc`,
        // Note that this order is correct as per the Rust function `codecs::merge`
        edited,
        original
      )

      // Handle the results
      if (filesModified === null) {
        vscode.window.showInformationMessage('Merge cancelled.')
        return
      }
      if (filesModified.length === 0) {
        vscode.window.showInformationMessage(
          'File merged successfully but no changes were detected.'
        )
        return
      }

      // Track files that couldn't be opened
      const failedFiles: string[] = []

      // Open each modified file to show git diff in the editor
      for (const filePath of filesModified) {
        try {
          const fileUri = vscode.Uri.file(filePath)

          // Check if file exists
          try {
            await vscode.workspace.fs.stat(fileUri)
          } catch {
            failedFiles.push(filePath)
            continue
          }

          // Open the file - VSCode will automatically show git decorations
          // and the user can use the Source Control view or gutter indicators
          // to see the changes
          await vscode.window.showTextDocument(fileUri, {
            preview: false,
            preserveFocus: false,
          })
        } catch (error) {
          failedFiles.push(filePath)
        }
      }

      // Show success message with count
      const successCount = filesModified.length - failedFiles.length
      if (successCount > 0) {
        vscode.window.showInformationMessage(
          `Merge completed. ${successCount} file${successCount === 1 ? '' : 's'} modified.`
        )
      }

      // Show warning for files that couldn't be opened
      if (failedFiles.length > 0) {
        vscode.window.showWarningMessage(
          `${failedFiles.length} file${failedFiles.length === 1 ? ' was' : 's were'} modified but could not be opened for diff view: ${failedFiles.join(', ')}`
        )
      }
    })
  )

  // Push document command
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.push-doc', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      // Save the document if it has unsaved changes
      if (editor.document.isDirty) {
        const saved = await editor.document.save()
        if (!saved) {
          vscode.window.showInformationMessage(
            'Push cancelled: document must be saved first.'
          )
          return
        }
      }

      const path = editor.document.uri.fsPath

      // Build options object
      const options: {
        target?: string
        force_new: boolean
        no_execute: boolean
        watch: boolean
        push_all_remotes: boolean
        args: string
      } = {
        force_new: false,
        no_execute: false,
        watch: false,
        push_all_remotes: true,
        args: ''
      }

      event('doc_push')

      try {
        const result = await vscode.commands.executeCommand<{
          url?: string
          status?: string
          message?: string
          success_count?: number
          fail_count?: number
          remotes?: Array<{service: string, url: string}>
        }>(`stencila.push-doc`, path, options)

        // Handle multiple remotes success case (handled by Rust)
        if (result?.status === 'success_multiple') {
          // Success message already shown by Rust LSP
          return
        }

        // Handle no remotes case
        if (result?.status === 'no_remotes') {
          // Prompt user to select a service to create new document
          const serviceChoice = await vscode.window.showQuickPick(
            [
              { label: 'Google Docs', value: 'gdoc', description: 'Create a new Google Doc' },
              { label: 'Microsoft 365', value: 'm365', description: 'Create a new Microsoft 365 document' },
              { label: 'Enter URL manually', value: 'manual', description: 'Connect to or update an existing remote' }
            ],
            {
              title: 'Select service to push to',
              placeHolder: 'No remotes tracked. Choose where to push this document.'
            }
          )

          if (serviceChoice) {
            let newTarget = serviceChoice.value
            let forceNew = true

            if (serviceChoice.value === 'manual') {
              const urlInput = await vscode.window.showInputBox({
                title: 'Enter remote URL',
                placeHolder: 'https://docs.google.com/document/d/...',
                prompt: 'Enter the full URL of the remote document'
              })
              if (!urlInput) {
                return
              }
              newTarget = urlInput
              forceNew = false // Don't force new when user provides explicit URL
            }

            // Retry with selected service or URL
            options.target = newTarget
            options.force_new = forceNew

            const retryResult = await vscode.commands.executeCommand<{url?: string}>(
              `stencila.push-doc`,
              path,
              options
            )

            if (retryResult?.url) {
              const choice = await vscode.window.showInformationMessage(
                `Successfully pushed to ${retryResult.url}`,
                'Open Remote',
                'Copy URL'
              )

              if (choice === 'Open Remote') {
                vscode.env.openExternal(vscode.Uri.parse(retryResult.url))
              } else if (choice === 'Copy URL') {
                vscode.env.clipboard.writeText(retryResult.url)
                vscode.window.showInformationMessage('URL copied to clipboard')
              }
            }
          }
          return
        }

        // Handle success case
        if (result?.url) {
          const choice = await vscode.window.showInformationMessage(
            `Successfully pushed to ${result.url}`,
            'Open Remote',
            'Copy URL'
          )

          if (choice === 'Open Remote') {
            vscode.env.openExternal(vscode.Uri.parse(result.url))
          } else if (choice === 'Copy URL') {
            vscode.env.clipboard.writeText(result.url)
            vscode.window.showInformationMessage('URL copied to clipboard')
          }
          return
        }

        // If we get here, something unexpected happened
        vscode.window.showErrorMessage('Push failed: no result returned')
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        vscode.window.showErrorMessage(`Push failed: ${errorMessage}`)
      }
    })
  )

  // Push document to specific remote command
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.push-doc-remote', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      // Save the document if it has unsaved changes
      if (editor.document.isDirty) {
        const saved = await editor.document.save()
        if (!saved) {
          vscode.window.showInformationMessage(
            'Push cancelled: document must be saved first.'
          )
          return
        }
      }

      const path = editor.document.uri.fsPath

      // Build options with push_all_remotes: false to get list of remotes
      const options: {
        target?: string
        force_new: boolean
        no_execute: boolean
        watch: boolean
        push_all_remotes: boolean
        args: string
      } = {
        force_new: false,
        no_execute: false,
        watch: false,
        push_all_remotes: false,
        args: ''
      }

      event('doc_push_specific')

      try {
        const result = await vscode.commands.executeCommand<{
          url?: string
          status?: string
          message?: string
          remotes?: Array<{service: string, url: string}>
        }>(`stencila.push-doc`, path, options)

        // Handle multiple remotes - let user choose one
        if (result?.status === 'multiple_remotes' && result.remotes) {
          const remoteItems = result.remotes.map((remote) => ({
            label: remote.service,
            description: remote.url,
            url: remote.url
          }))

          const selected = await vscode.window.showQuickPick(remoteItems, {
            title: 'Select remote to push to',
            placeHolder: 'Choose which remote to update.'
          })

          if (selected) {
            // Push to selected remote
            const retryOptions = {
              target: selected.url,
              force_new: false,
              no_execute: false,
              watch: false,
              push_all_remotes: false,
              args: ''
            }

            const retryResult = await vscode.commands.executeCommand<{url?: string}>(
              `stencila.push-doc`,
              path,
              retryOptions
            )

            if (retryResult?.url) {
              const choice = await vscode.window.showInformationMessage(
                `Successfully pushed to ${retryResult.url}`,
                'Open Remote',
                'Copy URL'
              )

              if (choice === 'Open Remote') {
                vscode.env.openExternal(vscode.Uri.parse(retryResult.url))
              } else if (choice === 'Copy URL') {
                vscode.env.clipboard.writeText(retryResult.url)
                vscode.window.showInformationMessage('URL copied to clipboard')
              }
            }
          }
          return
        }

        // Handle no remotes case
        if (result?.status === 'no_remotes') {
          vscode.window.showErrorMessage(
            'No remotes found. Use "Push document to remote" to create a new remote first.'
          )
          return
        }

        // If there's only one remote, just push to it
        if (result?.url) {
          const choice = await vscode.window.showInformationMessage(
            `Successfully pushed to ${result.url}`,
            'Open Remote',
            'Copy URL'
          )

          if (choice === 'Open Remote') {
            vscode.env.openExternal(vscode.Uri.parse(result.url))
          } else if (choice === 'Copy URL') {
            vscode.env.clipboard.writeText(result.url)
            vscode.window.showInformationMessage('URL copied to clipboard')
          }
          return
        }

        // If we get here, something unexpected happened
        vscode.window.showErrorMessage('Push failed: no result returned')
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        vscode.window.showErrorMessage(`Push failed: ${errorMessage}`)
      }
    })
  )

  // Push all documents command
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.push-docs', async () => {
      const options = {
        no_execute: false,
        args: ''
      }

      event('doc_push_all')

      try {
        await vscode.window.withProgress(
          {
            location: vscode.ProgressLocation.Notification,
            title: 'Pushing all tracked documents...',
            cancellable: false
          },
          async () => {
            const result = await vscode.commands.executeCommand<{total: number, succeeded: number, failed: number}>(
              `stencila.push-docs`,
              options
            )

            if (result) {
              const message = `Push complete: ${result.succeeded} succeeded, ${result.failed} failed`
              if (result.failed > 0) {
                vscode.window.showWarningMessage(message)
              } else {
                vscode.window.showInformationMessage(message)
              }
            }
          }
        )
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        vscode.window.showErrorMessage(`Push all failed: ${errorMessage}`)
      }
    })
  )

  // Pull document command
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.pull-doc', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      // Save the document if it has unsaved changes
      if (editor.document.isDirty) {
        const saved = await editor.document.save()
        if (!saved) {
          vscode.window.showInformationMessage(
            'Pull cancelled: document must be saved first.'
          )
          return
        }
      }

      const path = editor.document.uri.fsPath

      // Ask user for merge preference
      const mergeChoice = await vscode.window.showQuickPick(
        [
          {
            label: 'Merge with local file (keep local changes)',
            value: 'merge'
          },
          {
            label: 'Replace local file (overwrite local changes)',
            value: 'replace'
          }
        ],
        {
          title: 'Pull Mode',
          placeHolder: 'How should the pulled content be applied?'
        }
      )

      if (!mergeChoice) {
        return
      }

      const merge = mergeChoice.value === 'merge'

      const options: {
        target?: string
        merge: boolean
      } = {
        merge
      }

      event('doc_pull')

      try {
        const result = await vscode.commands.executeCommand<{
          modified_files?: string[]
          status?: string
          message?: string
          remotes?: Array<{service: string, url: string}>
        }>(`stencila.pull-doc`, path, options)

        // Handle multiple remotes case
        if (result?.status === 'multiple_remotes' && result.remotes) {
          const remoteItems = result.remotes.map((remote) => ({
            label: remote.service,
            description: remote.url,
            url: remote.url
          }))

          const selected = await vscode.window.showQuickPick(remoteItems, {
            title: 'Select remote to pull from',
            placeHolder: 'Multiple remotes found. Choose which one to pull from.'
          })

          if (selected) {
            // Retry with selected remote
            const retryOptions = {
              target: selected.url,
              merge
            }

            const retryResult = await vscode.commands.executeCommand<{modified_files?: string[]}>(
              `stencila.pull-doc`,
              path,
              retryOptions
            )

            if (retryResult?.modified_files) {
              const modifiedFiles = retryResult.modified_files as string[]

              if (modifiedFiles.length === 0) {
                vscode.window.showInformationMessage('Pulled successfully, no changes detected.')
              } else {
                // Open modified files
                for (const filePath of modifiedFiles) {
                  try {
                    const fileUri = vscode.Uri.file(filePath)
                    await vscode.window.showTextDocument(fileUri, {
                      preview: false,
                      preserveFocus: false
                    })
                  } catch (error) {
                    // Ignore file open errors
                  }
                }

                vscode.window.showInformationMessage(
                  `Pulled successfully, ${modifiedFiles.length} file${modifiedFiles.length === 1 ? '' : 's'} modified`
                )
              }
            } else {
              vscode.window.showInformationMessage('Pull completed')
            }
          }
          return
        }

        // Handle no remotes case
        if (result?.status === 'no_remotes') {
          // Prompt user to enter URL to pull from
          const urlInput = await vscode.window.showInputBox({
            title: 'Enter remote URL to pull from',
            placeHolder: 'https://docs.google.com/document/d/... or https://...',
            prompt: 'No tracked remotes found. Enter the URL of the remote document to pull from.',
            validateInput: (value) => {
              if (!value) {
                return 'URL is required'
              }
              if (!value.startsWith('http://') && !value.startsWith('https://')) {
                return 'URL must start with http:// or https://'
              }
              return null
            }
          })

          if (urlInput) {
            // Retry with user-provided URL
            options.target = urlInput

            const retryResult = await vscode.commands.executeCommand<{modified_files?: string[]}>(
              `stencila.pull-doc`,
              path,
              options
            )

            if (retryResult?.modified_files) {
              const modifiedFiles = retryResult.modified_files as string[]

              if (modifiedFiles.length === 0) {
                vscode.window.showInformationMessage('Pulled successfully, no changes detected.')
              } else {
                // Open modified files
                for (const filePath of modifiedFiles) {
                  try {
                    const fileUri = vscode.Uri.file(filePath)
                    await vscode.window.showTextDocument(fileUri, {
                      preview: false,
                      preserveFocus: false
                    })
                  } catch (error) {
                    // Ignore file open errors
                  }
                }

                vscode.window.showInformationMessage(
                  `Pulled successfully, ${modifiedFiles.length} file${modifiedFiles.length === 1 ? '' : 's'} modified`
                )
              }
            } else {
              vscode.window.showInformationMessage('Pull completed')
            }
          }
          return
        }

        // Handle success case
        if (result?.modified_files) {
          const modifiedFiles = result.modified_files as string[]

          if (modifiedFiles.length === 0) {
            vscode.window.showInformationMessage('Pulled successfully, no changes detected.')
          } else {
            // Open modified files
            for (const filePath of modifiedFiles) {
              try {
                const fileUri = vscode.Uri.file(filePath)
                await vscode.window.showTextDocument(fileUri, {
                  preview: false,
                  preserveFocus: false
                })
              } catch (error) {
                // Ignore file open errors
              }
            }

            vscode.window.showInformationMessage(
              `Pulled successfully, ${modifiedFiles.length} file${modifiedFiles.length === 1 ? '' : 's'} modified`
            )
          }
          return
        }

        // If we get here, something unexpected happened
        vscode.window.showInformationMessage('Pull completed')
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        vscode.window.showErrorMessage(`Pull failed: ${errorMessage}`)
      }
    })
  )

  // Watch document command
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.watch-doc', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      // Save the document if it has unsaved changes
      if (editor.document.isDirty) {
        const saved = await editor.document.save()
        if (!saved) {
          vscode.window.showInformationMessage(
            'Watch cancelled: document must be saved first.'
          )
          return
        }
      }

      const path = editor.document.uri.fsPath

      // Prompt for watch direction
      const directionChoice = await vscode.window.showQuickPick(
        [
          {
            label: 'Bi-directional',
            value: 'bi',
            description: 'Sync changes in both directions'
          },
          {
            label: 'From remote only',
            value: 'from-remote',
            description: 'Only pull changes from remote to repository'
          },
          {
            label: 'To remote only',
            value: 'to-remote',
            description: 'Only push changes from repository to remote'
          }
        ],
        {
          title: 'Watch Direction',
          placeHolder: 'How should changes be synced?'
        }
      )

      if (!directionChoice) {
        return
      }

      const options = {
        direction: directionChoice.value
      }

      event('doc_watch')

      try {
        const result = await vscode.commands.executeCommand<{
          status?: string
          message?: string
          watch_id?: string
          remote_url?: string
          remotes?: Array<{service: string, url: string}>
        }>('stencila.watch-doc', path, options)

        // Handle multiple remotes - let user choose one
        if (result?.status === 'multiple_remotes' && result.remotes) {
          const remoteItems = result.remotes.map((remote) => ({
            label: remote.service,
            description: remote.url,
            url: remote.url
          }))

          const selected = await vscode.window.showQuickPick(remoteItems, {
            title: 'Select remote to watch',
            placeHolder: 'Choose which remote to watch for changes.'
          })

          if (selected) {
            // Retry with selected remote
            const retryOptions = {
              target: selected.url,
              direction: directionChoice.value
            }

            const retryResult = await vscode.commands.executeCommand<{
              status?: string
              watch_id?: string
              remote_url?: string
            }>('stencila.watch-doc', path, retryOptions)

            if (retryResult?.status === 'success') {
              vscode.window.showInformationMessage(
                `Now watching document on ${selected.label}`
              )
            }
          }
          return
        }

        // Handle no remotes case
        if (result?.status === 'no_remotes') {
          vscode.window.showErrorMessage(
            'No remotes found. Use "Push document to remote" to create a remote first.'
          )
          return
        }

        // Handle already watched case
        if (result?.status === 'already_watched') {
          vscode.window.showInformationMessage('Document is already being watched.')
          return
        }

        // Handle success case
        if (result?.status === 'success') {
          vscode.window.showInformationMessage('Document is now being watched.')
          return
        }

        // If we get here, something unexpected happened
        vscode.window.showWarningMessage('Watch setup may not have completed successfully.')
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        vscode.window.showErrorMessage(`Watch failed: ${errorMessage}`)
      }
    })
  )

  // Unwatch document command
  context.subscriptions.push(
    vscode.commands.registerCommand('stencila.invoke.unwatch-doc', async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      const path = editor.document.uri.fsPath

      // Show confirmation dialog
      const confirmed = await vscode.window.showWarningMessage(
        'Stop watching this document? (Link to remote will be preserved)',
        { modal: true },
        'Stop Watching'
      )

      if (confirmed !== 'Stop Watching') {
        return
      }

      const options = {}

      event('doc_unwatch')

      try {
        const result = await vscode.commands.executeCommand<{
          status?: string
          message?: string
          remotes?: Array<{service: string, url: string}>
        }>('stencila.unwatch-doc', path, options)

        // Handle multiple watched remotes - let user choose one
        if (result?.status === 'multiple_watched' && result.remotes) {
          const remoteItems = result.remotes.map((remote) => ({
            label: remote.service,
            description: remote.url,
            url: remote.url
          }))

          const selected = await vscode.window.showQuickPick(remoteItems, {
            title: 'Select remote to unwatch',
            placeHolder: 'Choose which remote to stop watching.'
          })

          if (selected) {
            // Retry with selected remote
            const retryOptions = {
              target: selected.url
            }

            const retryResult = await vscode.commands.executeCommand<{
              status?: string
            }>('stencila.unwatch-doc', path, retryOptions)

            if (retryResult?.status === 'success') {
              vscode.window.showInformationMessage(
                `Stopped watching ${selected.label}`
              )
            }
          }
          return
        }

        // Handle not watched case
        if (result?.status === 'not_watched') {
          vscode.window.showInformationMessage('Document is not being watched.')
          return
        }

        // Handle success case
        if (result?.status === 'success') {
          vscode.window.showInformationMessage('Stopped watching document.')
          return
        }

        // If we get here, something unexpected happened
        vscode.window.showWarningMessage('Unwatch may not have completed successfully.')
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        vscode.window.showErrorMessage(`Unwatch failed: ${errorMessage}`)
      }
    })
  )

  // Document preview panel
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.view-doc',
      async (_docUri, _nodeType) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        event('doc_preview', { format: editor.document.languageId })

        await createDocumentViewPanel(context, editor.document.uri, editor)
      }
    )
  )
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.view-node',
      async (_docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        await createNodeViewPanel(
          context,
          editor.document.uri,
          null,
          nodeType,
          nodeId
        )
      }
    )
  )
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'stencila.view-node-authors',
      async (_docUri, nodeType, nodeId) => {
        const editor = vscode.window.activeTextEditor ?? lastTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        await createNodeViewPanel(
          context,
          editor.document.uri,
          null,
          nodeType,
          nodeId,
          true
        )
      }
    )
  )

  // Create a temporary document chat
  //
  // The new chat will be anchored at the end of the document
  context.subscriptions.push(
    vscode.commands.registerCommand(`stencila.chat-doc`, async () => {
      const editor = vscode.window.activeTextEditor ?? lastTextEditor
      if (!editor) {
        vscode.window.showErrorMessage('No active editor')
        return
      }

      event('doc_chat', { format: editor.document.languageId })

      const chatId = await vscode.commands.executeCommand<string>(
        'stencila.create-chat',
        editor.document.uri.toString(),
        null, // range
        'Discuss', // instruction type
        null, // node type
        'document'
      )

      await createNodeViewPanel(
        context,
        editor.document.uri,
        null,
        'Chat',
        chatId
      )
    })
  )

  // Create a temporary chat in the current document
  //
  // If the instruction type is not supplied it is inferred from the selected node
  // types (if any).
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.invoke.create-chat`,
      async ({ instructionType, nodeType, prompt, executeChat } = {}) => {
        const editor = vscode.window.activeTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        event('doc_chat_create', {
          format: editor.document.languageId,
          instructionType,
          nodeType,
          prompt,
          executeChat,
        })

        const chatId = await vscode.commands.executeCommand<string>(
          'stencila.create-chat',
          editor.document.uri.toString(),
          editor.selection,
          instructionType,
          nodeType,
          prompt,
          executeChat
        )

        await createNodeViewPanel(
          context,
          editor.document.uri,
          editor.selection.active,
          'Chat',
          chatId
        )
      }
    )
  )

  // Typed wrapper to the `invoke.create-chat` command for convenience
  // of following commands
  async function insertChat(options: {
    instructionType: 'Create' | 'Edit' | 'Fix';
    nodeType?: string;
    prompt?: string;
    executeChat?: boolean;
  }) {
    await vscode.commands.executeCommand(
      'stencila.invoke.create-chat',
      options
    )
  }

  // Create a `Create` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat.suggest`,
      async () => await insertChat({ instructionType: 'Create' })
    )
  )
  for (const prompt of [
    'code-chunk',
    'figure-code',
    'figure-flowchart',
    'figure-mermaid',
    'figure-svg',
    'figure-timeline',
    'list-ordered',
    'list-unordered',
    'math-block',
    'paragraph',
    'quote-block',
    'table-code',
    'table-empty',
    'table-filled',
  ]) {
    context.subscriptions.push(
      vscode.commands.registerCommand(
        `stencila.insert-chat.create.${prompt}`,
        async () =>
          await insertChat({
            instructionType: 'Create',
            // Do not need to prefix prompt with `stencila/create` because
            // providing instruction type
            prompt,
          })
      )
    )
  }

  // Create a `Edit` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat.edit`,
      async () => await insertChat({ instructionType: 'Edit' })
    )
  )

  // Create a `Fix` chat but do not run it straightaway
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-chat.fix`,
      async () => await insertChat({ instructionType: 'Fix' })
    )
  )

  // Insert a `create` command
  context.subscriptions.push(
    vscode.commands.registerCommand(
      `stencila.insert-command-create`,
      async () => {
        const editor = vscode.window.activeTextEditor
        if (!editor) {
          vscode.window.showErrorMessage('No active editor')
          return
        }

        let prompt = null
        const item: { prompt: { id: string } } =
          await vscode.commands.executeCommand(
            'stencila.prompts.menu',
            'Create'
          )
        if (item) {
          prompt = item.prompt.id
        }

        let message = await vscode.window.showInputBox({
          title: 'Instructions',
          placeHolder:
            "Describe what you want created (end with '...' for more options)",
          ignoreFocusOut: true,
        })

        let models = null
        if (message?.endsWith('...')) {
          message = message.slice(0, -3)

          const items: { model: { id: string } }[] =
            await vscode.commands.executeCommand('stencila.models.picker')
          if (items && items.length > 0) {
            models = items.map((item) => item.model.id)
          }
        }

        const nodeId = await vscode.commands.executeCommand<string>(
          'stencila.insert-node',
          editor.document.uri.toString(),
          editor.selection.active,
          'InstructionBlock',
          'Create',
          prompt,
          message,
          models
        )

        await vscode.commands.executeCommand(
          'stencila.run-node',
          editor.document.uri.toString(),
          'InstructionBlock',
          nodeId
        )
      }
    )
  )
}
