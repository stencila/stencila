import { existsSync, writeFileSync, mkdirSync } from "fs";
import path from "path";

import * as vscode from "vscode";

import { nodeColors } from "./node-colours";

/**
 * A map of gutter decoration types
 *
 * Allows for caching or arbitrary gutter decorations for various combinations
 * of node type nestings and states.
 */
const gutterDecorations: Record<string, vscode.TextEditorDecorationType> = {};

/**
 * Get the gutter decoration type for a line spanned by certain node types/states
 *
 * Caches the created SVG icons on file (they need to have a path anyway) and
 * the `TextEditorDecorationType` into `gutterDecorations`.
 */
export function gutterDecorationType(
  context: vscode.ExtensionContext,
  nodes: string[],
  markerWidth: number
): vscode.TextEditorDecorationType {
  // Unique key for this set of node types/states. Must include marker width
  // since that define uniqueness.
  const key = `${nodes.join(".")}.w${markerWidth}`;

  // Check in-memory cache
  const existing = gutterDecorations[key];
  if (existing) {
    return existing;
  }

  const storageUri = context.storageUri || context.globalStorageUri;
  const cacheDir = path.join(storageUri.fsPath, "gutter-icons");
  const iconPath = path.join(cacheDir, `${key}.svg`);

  // Create the SVG icon and save to cache if it does not yet exist
  if (!existsSync(iconPath)) {
    // Create SVG
    // Uses a default opacity aimed to work well in both light and dark themes
    // for more node colors. For running nodes pulses opacity up to 1 and down to 0.2;
    let svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <style>
    .default {
      opacity: 0.5;
    }
    .running {
      animation: pulse 2s infinite;
    }
    @keyframes pulse {
      0% { opacity: 1; }
      50% { opacity: 0.2; }
      100% { opacity: 1; }
    }
  </style>
`;

    // Add vertical bars for each node type with start and/or end "tabs"
    // and animation if running
    const width = markerWidth;
    const height = markerWidth;
    const gap = markerWidth;
    let x = 0;
    for (const node of nodes) {
      let type = node;
      let start = false;
      let end = false;
      let style = "default";
      if (node.includes("_start")) {
        type = type.replace("_start", "");
        start = true;
      }
      if (node.includes("_end")) {
        type = type.replace("_end", "");
        end = true;
      }
      if (node.includes("_running")) {
        type = type.replace("_running", "");
        style = "running";
      }

      // Use the darkest node color ('textColor')
      const color: string = nodeColors[type][2] ?? "#000000";

      svg += `  <rect x="${x}" y="0" width="${width}" height="24" fill="${color}" class="${style}" />\n`;

      if (start) {
        svg += `  <rect x="${x + width}" y="0" width="${gap}" height="${height}" fill="${color}" class="${style}" />\n`;
      }

      if (end) {
        svg += `  <rect x="${x + width}" y="${24 - height}" width="${gap}" height="${height}" fill="${color}" class="${style}" />\n`;
      }

      x += width + gap;
    }

    svg += "</svg>\n";

    // Save the SVG to the cache
    if (!existsSync(cacheDir)) {
      mkdirSync(cacheDir, { recursive: true });
    }
    writeFileSync(iconPath, svg, "utf-8");
  }

  // Create the decoration using the icon
  const decoration = vscode.window.createTextEditorDecorationType({
    gutterIconPath: iconPath,
    gutterIconSize: "cover",
  });

  // Cache the decoration
  gutterDecorations[key] = decoration;

  return decoration;
}
