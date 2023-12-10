import { DocumentView } from "../types";

export const VIEWS: Record<DocumentView, string> = {
  static: "Fixed, read-only view",
  live: "Live updating view",
  dynamic: "Live updating and interactive view",
  source: "Source code view",
  split: "Two panel split view",
  visual: "Visual editor",
};
