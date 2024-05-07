import * as vscode from "vscode";

export const pendingDecoration = vscode.window.createTextEditorDecorationType({
  // TODO: set colors to greyish
  // Grey indicator in overview ruler
  // Use the right lane because it is for diagnostics
  overviewRulerColor: "#28a31f",
  overviewRulerLane: vscode.OverviewRulerLane.Right,

  // Styling for the inline text
  dark: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 5%)",
    },
  },
  light: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 95%)",
    },
  },
});

export const runningDecoration = vscode.window.createTextEditorDecorationType({
  // TODO: set colors to blueish
  // Blue indicator in overview ruler
  // Use the right lane because it is for diagnostics
  overviewRulerColor: "#28a31f",
  overviewRulerLane: vscode.OverviewRulerLane.Right,

  // Styling for the inline text
  dark: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 5%)",
    },
  },
  light: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 95%)",
    },
  },
});

export const succeededDecoration = vscode.window.createTextEditorDecorationType({
  // TODO: check these colors
  // Green indicator in overview ruler
  // Use the right lane because it is for diagnostics
  overviewRulerColor: "#28a31f",
  overviewRulerLane: vscode.OverviewRulerLane.Right,

  // Styling for the inline text
  dark: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 5%)",
    },
  },
  light: {
    after: {
      color: "#28a31f",
      backgroundColor: "hsl(115deg 100% 95%)",
    },
  },
});
