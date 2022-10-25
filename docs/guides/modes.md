A core feature of Stencila, and one that sets it apart from other similar solutions, is document _modes_. The mode determines how a document is presented to a user, how they can interact with it, and which parts, if any, they can modify.

Modes allow a single document, be it a Markdown file, a Jupyter Notebook, or a Microsoft Word file, to be accessed by different users in different ways. For example, you might set the mode to **Static** for anonymous users, but to **Write** for authors.

Under the hood, modes influence aspects of how a document is deployed.

1. **HTML**: What information the HTML generated for your document contains

2. **JavaScript**: The size and capabilities of the JavaScript that is loaded in the browser

3. **Patches**: Which patches are permitted by the Stencila document server (some modes allow node, some ignore certain patches, some modes allow all patches)

4. **Forking**: Whether documents are forked for each user

5. **Merging**: Whether changes to forked documents are merged back into the main document

# Static

Only includes web components necessary for reading a static page of the document.

# Dynamic

Adds the client so that patches can be received (but not sent) so that
the page represents a "live view" of the document.

# Inspect

Adds web components necessary for inspecting (and modifying) the
execution of a document (e.g. viewing code of code chunks)

# Interact

Adds web components necessary for interacting with the document
but not for inspecting of modifying its execution.

# Alter

# Develop

In this mode the user can edit the dynamic elements of the document (i.e. create, update, and
delete code nodes) but can not modify the static elements.

# Edit

In this mode the user can edit the content of the document (i.e. create, update, and
delete non-code nodes) but can not alter the code nodes (but can inspect it).

# Write

Sets the mode to `Write` to allow user to modify code and content.

# Code

# Shell
