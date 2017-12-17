# Projects - Iteration I

## Goal

Implement `ProjectComponent` and prepare `DocumentContainer`, an interface for multi-document manipulation.

## Tasks

- Develop DocumentContainer interface
- Develop ManifestDocument
- Render a ContextSection (sidebar) which can be enabled and disabled and hold arbitrary content (e.g. function help or issues list)
- When changing tabs the active context should be preserved (e.g. open function help for `sum` in a sheet then switch to a document -> help for sum should still be open)

### DocumentContainer

Maintains a registry of document sessions within one project.

```js
let documentContainer = new DocumentContainer()
dc.addSession('manifest', manifestSession)
dc.addSession('pub-meta', pubMetaSession)
dc.addSession('manuscript-1', docSession1)
dc.addSession('sheet-1', sheetSession1)
dc.addSession('notebook-1', docSession2)

$$(ProjectComponent, {
  documentContainer
})
```


### ManifestDocument

We go with JSON for now, but this could also be XML.

```js
{
  documents: [
    { "type": "manuscript", "src": "manuscript-1.xml" name: "Climate Change Report"},
    { "type": "sheet", "src": "sheet-1.xml" name: "Data"},
    { "type": "notebook", "src": "notebook-1.xml" name: "Methods"}
  ]
}
```

### ProjectComponent

Consists of a project bar, a context section and the active editor (either sheet or document). It takes a documentContainer object which holds multiple editor sessions (one per document + a manifest session). Here's the proposed structure:

```
ProjectComponent
  - .se-main-pane
    - .se-editor-pane
      - SheetEditor | DocumentEditor
    - EditorContextPane
    - ProjectBar
      - ProjectTabs
      - ToggleProjectIssues
      - ToggleProjectHelp
```

### ProjectBar

- Issues Status
  - aggregated information for all issues of a document
  - when clicked opens the
- Tabs for navigating between documents
- Button to create a new document
- Toggle for help dialog
