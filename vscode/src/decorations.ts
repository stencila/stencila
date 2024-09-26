import * as vscode from "vscode";

const unexecuted = "0deg";
const unexecutedMid = `hsl(${unexecuted} 0% 50%)`;
const unexecutedLight = `hsl(${unexecuted} 0% 95%)`;
const unexecutedDark = `hsl(${unexecuted} 0% 5%)`;

export const unexecutedDecoration =
  vscode.window.createTextEditorDecorationType({
    overviewRulerColor: unexecutedMid,
    overviewRulerLane: vscode.OverviewRulerLane.Right,
    light: {
      after: {
        color: unexecutedMid,
        backgroundColor: unexecutedLight,
      },
    },
    dark: {
      after: {
        color: unexecutedMid,
        backgroundColor: unexecutedDark,
      },
    },
  });

const stale = "45deg";
const staleMid = `hsl(${stale} 100% 50%)`;
const staleLight = `hsl(${stale} 100% 95%)`;
const staleDark = `hsl(${stale} 100% 5%)`;

export const staleDecoration = vscode.window.createTextEditorDecorationType({
  overviewRulerColor: staleMid,
  overviewRulerLane: vscode.OverviewRulerLane.Right,
  light: {
    after: {
      color: staleMid,
      backgroundColor: staleLight,
    },
  },
  dark: {
    after: {
      color: staleMid,
      backgroundColor: staleDark,
    },
  },
});

const pending = "180deg";
const pendingMid = `hsl(${pending} 100% 30%)`;
const pendingLight = `hsl(${pending} 100% 95%)`;
const pendingDark = `hsl(${pending} 100% 5%)`;

export const pendingDecoration = vscode.window.createTextEditorDecorationType({
  overviewRulerColor: pendingMid,
  overviewRulerLane: vscode.OverviewRulerLane.Right,
  light: {
    after: {
      color: pendingMid,
      backgroundColor: pendingLight,
    },
  },
  dark: {
    after: {
      color: pendingMid,
      backgroundColor: pendingDark,
    },
  },
});

const running = "230deg";
const runningMid = `hsl(${running} 100% 50%)`;
const runningLight = `hsl(${running} 100% 95%)`;
const runningDark = `hsl(${running} 100% 5%)`;

export const runningDecoration = vscode.window.createTextEditorDecorationType({
  overviewRulerColor: runningMid,
  overviewRulerLane: vscode.OverviewRulerLane.Right,
  light: {
    after: {
      color: runningMid,
      backgroundColor: runningLight,
    },
  },
  dark: {
    after: {
      color: runningMid,
      backgroundColor: runningDark,
    },
  },
});

const skipped = "170deg";
const skippedMid = `hsl(${skipped} 100% 30%)`;
const skippedLight = `hsl(${skipped} 100% 95%)`;
const skippedDark = `hsl(${skipped} 100% 5%)`;

export const skippedDecoration = vscode.window.createTextEditorDecorationType(
  {
    overviewRulerColor: skippedMid,
    overviewRulerLane: vscode.OverviewRulerLane.Right,
    light: {
      after: {
        color: skippedMid,
        backgroundColor: skippedLight,
      },
    },
    dark: {
      after: {
        color: skippedMid,
        backgroundColor: skippedDark,
      },
    },
  }
);

const succeeded = "120deg";
const succeededMid = `hsl(${succeeded} 100% 30%)`;
const succeededLight = `hsl(${succeeded} 100% 95%)`;
const succeededDark = `hsl(${succeeded} 100% 5%)`;

export const succeededDecoration = vscode.window.createTextEditorDecorationType(
  {
    overviewRulerColor: succeededMid,
    overviewRulerLane: vscode.OverviewRulerLane.Right,
    light: {
      after: {
        color: succeededMid,
        backgroundColor: succeededLight,
      },
    },
    dark: {
      after: {
        color: succeededMid,
        backgroundColor: succeededDark,
      },
    },
  }
);

const succeededFork = "160deg";
const succeededForkMid = `hsl(${succeededFork} 90% 30%)`;
const succeededForkLight = `hsl(${succeededFork} 90% 95%)`;
const succeededForkDark = `hsl(${succeededFork} 90% 5%)`;

export const succeededForkDecoration = vscode.window.createTextEditorDecorationType(
  {
    overviewRulerColor: succeededForkMid,
    overviewRulerLane: vscode.OverviewRulerLane.Right,
    light: {
      after: {
        color: succeededForkMid,
        backgroundColor: succeededForkLight,
      },
    },
    dark: {
      after: {
        color: succeededForkMid,
        backgroundColor: succeededForkDark,
      },
    },
  }
);

const active = "260deg";
const activeMid = `hsl(${active} 100% 80%)`;

export const activeDecoration = vscode.window.createTextEditorDecorationType({
  overviewRulerColor: activeMid,
  overviewRulerLane: vscode.OverviewRulerLane.Right,
  light: {
    after: {
      color: activeMid,
      backgroundColor: `hsl(${active} 100% 50% / 5%)`,
    },
  },
  dark: {
    after: {
      color: activeMid,
      backgroundColor: `hsl(${active} 100% 50% / 30%)`,
    },
  },
});
