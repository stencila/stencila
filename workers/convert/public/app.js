const MAX_SIZE = 25 * 1024 * 1024;

const JSON_FORMATS = new Set([
  "atproto.json",
  "cytoscape",
  "echarts",
  "ipynb",
  "json",
  "jsonld",
  "json5",
  "lexical",
  "oxa",
  "oxa.json",
  "plotly",
  "tiptap",
  "vegalite",
]);
const YAML_FORMATS = new Set(["oxa.yaml", "yaml"]);

const form = document.querySelector("#convert-form");
const outputStatus = document.querySelector("#output-status");
const outputStatusText = document.querySelector("#output-status-text");
const dropZone = document.querySelector("#drop-zone");
const dropTitle = document.querySelector("#drop-title");
const dropMeta = document.querySelector("#drop-meta");
const fileInput = document.querySelector("#file-input");
const urlField = document.querySelector("#url-field");
const urlInput = document.querySelector("#url-input");
const fromSelect = document.querySelector("#from-select");
const toSelect = document.querySelector("#to-select");
const copyButton = document.querySelector("#copy-button");
const downloadButton = document.querySelector("#download-button");
const outputDownloadButton = document.querySelector("#output-download-button");
const previewCode = document.querySelector("#preview-code");
const treeView = document.querySelector("#tree-view");
const treeTab = document.querySelector("#tree-tab");
const outputName = document.querySelector("#output-name");
const sourceTabs = document.querySelectorAll("[data-source]");
const outputTabs = document.querySelectorAll("[data-output-tab]");
const outputPanes = document.querySelectorAll("[data-output-pane]");

let sourceMode = "file";
let currentBlob;
let currentText = "";
let currentFilename = "converted.txt";

loadFormats();

sourceTabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    sourceMode = tab.dataset.source;
    sourceTabs.forEach((item) => item.classList.toggle("active", item === tab));
    dropZone.classList.toggle("hidden", sourceMode !== "file");
    urlField.classList.toggle("hidden", sourceMode !== "url");
    setStatus("Ready");
  });
});

outputTabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    if (!tab.disabled) {
      setActiveOutputTab(tab.dataset.outputTab);
    }
  });
});

fileInput.addEventListener("change", () => {
  const file = fileInput.files?.[0];
  if (!file) {
    dropTitle.textContent = "Select file";
    dropMeta.textContent = "25 MiB maximum";
    return;
  }
  dropTitle.textContent = file.name;
  dropMeta.textContent = formatSize(file.size);
});

dropZone.addEventListener("dragover", (event) => {
  event.preventDefault();
  dropZone.classList.add("dragging");
});

dropZone.addEventListener("dragleave", () => {
  dropZone.classList.remove("dragging");
});

dropZone.addEventListener("drop", (event) => {
  event.preventDefault();
  dropZone.classList.remove("dragging");
  const file = event.dataTransfer?.files?.[0];
  if (file) {
    fileInput.files = event.dataTransfer.files;
    dropTitle.textContent = file.name;
    dropMeta.textContent = formatSize(file.size);
  }
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  await convert();
});

copyButton.addEventListener("click", async () => {
  if (!currentText) {
    return;
  }
  await navigator.clipboard.writeText(currentText);
  setStatus("Copied");
});

downloadButton.addEventListener("click", downloadOutput);
outputDownloadButton.addEventListener("click", downloadOutput);

function downloadOutput() {
  if (!currentBlob) {
    return;
  }
  const link = document.createElement("a");
  link.href = URL.createObjectURL(currentBlob);
  link.download = currentFilename;
  link.click();
  URL.revokeObjectURL(link.href);
}

async function loadFormats() {
  try {
    const formatsUrl = new URL("/api/formats", window.location.origin);
    const pageParams = new URLSearchParams(window.location.search);
    for (const name of ["from", "to"]) {
      if (pageParams.has(name)) {
        formatsUrl.searchParams.set(name, pageParams.get(name));
      }
    }

    const response = await fetch(formatsUrl);
    if (!response.ok) {
      throw new Error("Formats unavailable");
    }

    const formats = await response.json();
    const readable = formats
      .filter((format) => format.from)
      .sort((left, right) => left.label.localeCompare(right.label));
    const writable = formats
      .filter((format) => format.to)
      .sort((left, right) => left.label.localeCompare(right.label));

    for (const format of readable) {
      fromSelect.append(optionFor(format));
    }
    for (const format of writable) {
      toSelect.append(optionFor(format));
    }

    toSelect.value = writable.some((format) => format.name === "json")
      ? "json"
      : (writable[0]?.name ?? "");
  } catch (error) {
    setStatus(error.message, true);
  }
}

function optionFor(format) {
  const option = document.createElement("option");
  option.value = format.name;
  option.textContent = `${format.label} .${format.extension}`;
  return option;
}

async function convert() {
  setBusy(true);
  clearResult();

  try {
    const body = new FormData();
    const file = fileInput.files?.[0];
    const url = urlInput.value.trim();
    const outputFormat = toSelect.value;

    if (sourceMode === "file") {
      if (!file) {
        throw new Error("Select a file");
      }
      if (file.size > MAX_SIZE) {
        throw new Error("File exceeds 25 MiB");
      }
      body.append("file", file);
    } else {
      if (!url) {
        throw new Error("Enter a supported identifier or URL");
      }
      body.append("url", url);
    }

    body.append("to", outputFormat);
    if (fromSelect.value) {
      body.append("from", fromSelect.value);
    }
    for (const [id, field] of [
      ["compact", "compact"],
      ["embed-media", "embedMedia"],
    ]) {
      if (document.getElementById(id).checked) {
        body.append(field, "true");
      }
    }

    const response = await fetch("/api/convert", {
      method: "POST",
      body,
    });

    if (!response.ok) {
      throw new Error(await errorMessage(response));
    }

    currentFilename =
      response.headers.get("X-Stencila-Output-Filename") ?? "converted";
    currentBlob = await response.blob();
    const output = await outputPreview(
      currentBlob,
      response.headers,
      outputFormat,
    );
    currentText = output.text;

    outputName.textContent = currentFilename;
    renderSource(output.text, output.language);
    renderTree(output.tree);
    copyButton.disabled = !currentText;
    downloadButton.disabled = false;
    if (output.previewable) {
      setStatus("Complete");
    } else {
      showDownloadPrompt();
    }
  } catch (error) {
    const message = error.message || "Conversion failed";
    outputName.textContent = "";
    setStatus(message, true);
    renderTree(undefined);
  } finally {
    setBusy(false);
  }
}

async function outputPreview(blob, headers, format) {
  const contentType = normalizeContentType(headers.get("Content-Type") ?? "");
  const isText =
    contentType.startsWith("text/") ||
    contentType.includes("json") ||
    contentType.includes("xml") ||
    contentType.includes("yaml");

  if (!isText) {
    return {
      text: "",
      language: "language-none",
      tree: undefined,
      previewable: false,
    };
  }

  const rawText = stripBom(await blob.text());

  return {
    text: rawText,
    language: prismLanguage(format, contentType),
    tree: parseStructuredOutput(rawText, format, contentType),
    previewable: true,
  };
}

function parseStructuredOutput(text, format, contentType) {
  const normalized = format.toLowerCase();
  const jsonLike =
    JSON_FORMATS.has(normalized) ||
    contentType.includes("json") ||
    contentType.endsWith("+json");

  if (normalized === "json5") {
    return parseJson5(text);
  }

  if (jsonLike) {
    return parseJson(text) ?? parseJson5(text);
  }

  if (YAML_FORMATS.has(normalized) || contentType.includes("yaml")) {
    return parseYaml(text);
  }

  return undefined;
}

function parseJson(text) {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

function parseJson5(text) {
  if (!window.JSON5) {
    return undefined;
  }

  try {
    return window.JSON5.parse(text);
  } catch {
    return undefined;
  }
}

function parseYaml(text) {
  if (!window.jsyaml) {
    return undefined;
  }

  try {
    return window.jsyaml.load(text);
  } catch {
    return undefined;
  }
}

function normalizeContentType(contentType) {
  return contentType.split(";")[0].trim().toLowerCase();
}

function stripBom(text) {
  return text.charCodeAt(0) === 0xfeff ? text.slice(1) : text;
}

function prismLanguage(format, contentType) {
  const normalized = format.toLowerCase();

  if (normalized === "json5") {
    return "language-json5";
  }
  if (JSON_FORMATS.has(normalized) || contentType.includes("json")) {
    return "language-json";
  }
  if (YAML_FORMATS.has(normalized) || contentType.includes("yaml")) {
    return "language-yaml";
  }
  if (normalized === "md" || normalized === "markdown") {
    return "language-markdown";
  }
  if (normalized === "html" || contentType.includes("html")) {
    return "language-markup";
  }

  return "language-none";
}

function renderSource(text, language) {
  previewCode.className = language;
  previewCode.textContent = text;

  if (window.Prism) {
    window.Prism.highlightElement(previewCode);
  }
}

function renderTree(value) {
  treeView.textContent = "";

  if (value === undefined) {
    treeTab.disabled = true;
    setActiveOutputTab("source");
    return;
  }

  treeTab.disabled = false;
  treeView.append(treeNode("root", value));
  setActiveOutputTab("tree");
}

function treeNode(label, value) {
  if (Array.isArray(value)) {
    return branchNode(label, value, `${value.length} items`);
  }

  if (value && typeof value === "object") {
    return branchNode(label, value, `${Object.keys(value).length} keys`);
  }

  const row = document.createElement("div");
  row.className = "tree-leaf";
  row.append(treeKey(label), treeValue(value));
  return row;
}

function branchNode(label, value, meta) {
  const details = document.createElement("details");
  details.className = "tree-node";
  details.open = true;

  const summary = document.createElement("summary");
  summary.append(treeKey(label), treeMeta(meta));
  details.append(summary);

  const children = document.createElement("div");
  children.className = "tree-children";

  const entries = Array.isArray(value)
    ? value.entries()
    : Object.entries(value);
  for (const [key, child] of entries) {
    children.append(treeNode(String(key), child));
  }

  details.append(children);
  return details;
}

function treeKey(label) {
  const key = document.createElement("span");
  key.className = "tree-key";
  key.textContent = label;
  return key;
}

function treeMeta(text) {
  const meta = document.createElement("span");
  meta.className = "tree-meta";
  meta.textContent = text;
  return meta;
}

function treeValue(value) {
  const item = document.createElement("span");
  item.className = `tree-value tree-value-${valueType(value)}`;
  item.textContent = value === null ? "null" : JSON.stringify(value);
  return item;
}

function valueType(value) {
  if (value === null) {
    return "null";
  }
  if (Array.isArray(value)) {
    return "array";
  }
  return typeof value;
}

async function errorMessage(response) {
  try {
    const json = await response.json();
    return json.error?.message ?? `Request failed: ${response.status}`;
  } catch {
    return `Request failed: ${response.status}`;
  }
}

function clearResult() {
  currentBlob = undefined;
  currentText = "";
  copyButton.disabled = true;
  downloadButton.disabled = true;
  outputName.textContent = "Converting";
  renderSource("", "language-none");
  renderTree(undefined);
}

function setActiveOutputTab(name) {
  outputTabs.forEach((tab) => {
    tab.classList.toggle("active", tab.dataset.outputTab === name);
  });
  outputPanes.forEach((pane) => {
    pane.classList.toggle("hidden", pane.dataset.outputPane !== name);
  });
}

function setBusy(busy) {
  document.querySelector("#convert-button").disabled = busy;
  if (busy) {
    setStatus("Converting");
  }
}

function setStatus(message, isError = false) {
  const state = isError ? "error" : message === "Converting" ? "busy" : "ready";
  outputStatus.dataset.state = state;
  outputStatusText.textContent = isError ? message : "";
  outputStatus.classList.toggle("hidden", state === "ready");
}

function showDownloadPrompt() {
  outputStatus.dataset.state = "download";
  outputStatusText.textContent = "";
  outputStatus.classList.remove("hidden");
}

function formatSize(size) {
  if (size < 1024) {
    return `${size} B`;
  }
  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(1)} KiB`;
  }
  return `${(size / 1024 / 1024).toFixed(1)} MiB`;
}
