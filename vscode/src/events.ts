import * as vscode from "vscode";

import { posthog } from 'posthog-js'

const isTelemetryEnabled = vscode.env.isTelemetryEnabled;


if (isTelemetryEnabled) {
  posthog.init('LeXA_J7NbIow0-mEejPwazN7WvZCj-mFKSvLL5oM4w0', { api_host: 'https://events.stencila.cloud' })
}

export const event = isTelemetryEnabled
  ? (name: string, data?: any) => posthog.capture(name, data)
  : (name: string, data?: any) => {};