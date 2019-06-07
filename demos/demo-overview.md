This demo gives an overview of the Stencila command line tool.

# Checking it's installed

You can check which version of `stencila` is installed by using the `--version` option:

```bash
stencila --version
```

# Getting help

Use the `--help` option to get help at any time.

```bash pause=2
stencila --help
```

# Converting files

Let's get started with using Stencila's converters.

### You can convert between the following textual formats:

- docx (Microsoft Word), gdoc (Google Docs), html, jats, latex, md, odt, pdf

### And between the following tabular data formats:

- csv, ods, tdp (Tabular Data Package), xlsx

### Additional formats:

- rpng (reproducible pngs), yaml, pandoc, json5, json

## Example: Markdown to YAML

```bash pause=2
stencila convert ./examples/README.md ./examples/output/README-from-md.yml
```

Let's see what the converted YAML file looks like.
This is an example of the Stencila schema file in YAML format:

```bash pause=3
cat ./examples/output/README-from-md.yml
```

---

## Example: Markdown to DOCX (and back!)

Next, let's create a DOCX from the Markdown file.

```bash pause=2
stencila convert ./examples/README.md ./examples/output/README-from-md.docx
```

And open the file:

```bash pause=5
open ./examples/output/README-from-md.docx
```

After making some edits, and saving the DOCX file...
Let's convert it back to a Markdown file:

```bash pause=2
stencila convert ./examples/output/README-from-md.docx ./examples/output/README-from-docx.md
```

The Markdown file should be updated:

```bash pause=3
cat ./examples/output/README-from-docx.md
```

# Serving project folders

Serving a project folder allows you to view files in various formats
locally using a web browser.

In this demo, we're using `&` to run `stencila serve --sync` in the background.
You can just use `stencila serve [folder] --sync`.

The `--sync` option lets you view changes in the browser automatically
on file save.

```bash pause=5
stencila serve examples --sync &
```

We're now serving examples at https://localhost:3001

Without having to explictly convert files, you can view various formats
as HTML pages.

Let's open a Markdown file (the original README.md).

```bash pause=3
open http://localhost:3001/README.md
```

Now let's open the updated Markdown file (README-from-docx.md):

```bash pause=3
open http://localhost:3001/output/README-from-docx.md
```

As you make changes and save your file, you'll see the document
automatically update in the browser.

Thanks for watching!

```bash
lsof -ti:3000 | xargs kill
```
