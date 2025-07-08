import { PythonExtension } from '@vscode/python-extension'
import * as vscode from 'vscode'


/**
 * Register a listener to restart the LSP server if the Python environment is changed
 */
export async function registerPythonExtensionListener(
  context: vscode.ExtensionContext
) {
  const pythonApi: PythonExtension = await PythonExtension.api()

  const listener = pythonApi.environments.onDidChangeActiveEnvironmentPath(
    () => {
      vscode.commands.executeCommand('stencila.lsp-server.restart')
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
