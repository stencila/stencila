import { PythonExtension } from '@vscode/python-extension'
import * as vscode from 'vscode'

// Track path to prevent spurious restarts
let currentPythonPath: string | undefined

/**
 * Register a listener to restart the LSP server if the Python environment is changed
 */
export async function registerPythonExtensionListener(
  context: vscode.ExtensionContext
) {
  const pythonApi: PythonExtension = await PythonExtension.api()

  // Store initial path
  currentPythonPath = pythonApi.environments.getActiveEnvironmentPath()?.path

  const listener = pythonApi.environments.onDidChangeActiveEnvironmentPath(
    (uri) => {
      const newPath = uri?.path

      // Only restart if the path actually changed
      if (newPath !== currentPythonPath) {
        console.log(`Stencila: Python environment changed from "${currentPythonPath}" to "${newPath}", restarting LSP`)
        currentPythonPath = newPath
        vscode.commands.executeCommand('stencila.lsp-server.restart')
      }
    }
  )
  context.subscriptions.push(listener)
}

/**
 * Get environment variables for the LSP server related to Python
 *
 * PYTHON_PATH: the path to the Python interpreter selected via, or recommended by, the `ms-python.python` extension
 */
export async function getPythonEnvVars(): Promise<Record<string, string>> {
  const pythonApi: PythonExtension = await PythonExtension.api()
  const environmentPath = pythonApi.environments.getActiveEnvironmentPath()

  return environmentPath ? { PYTHON_PATH: environmentPath.path } : {}
}
