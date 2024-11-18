import * as vscode from 'vscode';
import * as path from 'path';
import * as os from 'os';

import { cliPath } from './cli';

/**
 * Register a command to open a shell with the Stencila CLI used by
 * VSCode on its PATH.
 */
export function registerStencilaShell(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('stencila.shell', () => {
        // Get the path to the directory containing the CLI binary
        const cliDir = path.dirname(cliPath(context));
        
        // Get the user's default shell
        const [shellPath, ...shellArgs] = getDefaultShell();
        
        // Create the environment variables for the terminal
        const env = Object.assign({}, process.env, {
            PATH: `${cliDir}${path.delimiter}${process.env.PATH}`
        });

        // Create terminal with modified PATH
        const terminal = vscode.window.createTerminal({
            name: 'Stencila Shell',
            message: 'This shell uses the same version of Stencila CLI as used by Visual Studio Code. Use `stencila -h` to get a list of commands.\n',
            shellPath,
            shellArgs,
            env
        });

        terminal.show();
    });

    context.subscriptions.push(disposable);
}

/**
 * Get the user's default shell, including any arguments
 */
function getDefaultShell(): string[] {
    const platform = os.platform();
    
    // Windows
    if (platform === 'win32') {
        // Try to use PowerShell Core first, fall back to Windows PowerShell
        const powershellPath = process.env.PWSHPATH || 'pwsh.exe';
        if (hasCommand(powershellPath)) {
            return [powershellPath, '-NoLogo'];
        }
        
        // Fall back to Command Prompt if PowerShell is not available
        return [process.env.COMSPEC || 'cmd.exe'];
    }
    
    // MacOS / Linux
    return [
        process.env.SHELL || '/bin/bash',
        '-l' // Login shell to ensure profile is loaded
    ];
}

/**
 * Check that a command is available on the machine
 */
function hasCommand(command: string): boolean {
    try {
        require('child_process').execSync(`${command} --version`, { stdio: 'ignore' });
        return true;
    } catch {
        return false;
    }
}
