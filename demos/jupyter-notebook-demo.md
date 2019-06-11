This demo shows how to convert between Jupyter Notebooks and several file formats.

# Checking it's installed

First, check to make sure `jupyter notebook` is installed by using the `--version` option:

```bash
jupyter notebook --version
```

You can check which version of `stencila` is installed by using the `--version` option:

```bash
stencila --version
```

# Open a Jupyter Notebook

For this demo, we'll be using Josh Hemann's `sunspots.ipynb` from https://github.com/jupyter/jupyter/wiki/A-gallery-of-interesting-Jupyter-Notebooks.

```bash pause=5
jupyter notebook examples/ipynb/sunspots.ipynb &
```

# Jupyter Notebook to DOCX

Run the `stencila convert` command to create a DOCX file from the Jupyter Notebook.

```bash pause=2
stencila convert examples/ipynb/sunspots.ipynb examples/output/sunspots.docx
```

And open the file:

```bash pause=5
open examples/output/sunspots.docx
```

## A brief note on Reproducible PNGs (rPNGs)

You'll notice that when viewing the converted DOCX file, code cells will be hidden, and only code cell outputs are visible. Stencila converters embed the input code cells in the ouput PNGs, which remain uneditable until the textual format is converted back to an executable format (e.g. ipynb).

# DOCX to YAML

After making some edits, and saving the DOCX file...
Let's convert it back to a YAML file (Stencila schema).

```bash pause=2
stencila convert examples/output/sunspots.docx examples/output/sunspots.yml
```

Viewing the converted file in YAML can be useful for debugging.

```bash pause=3
cat examples/output/sunspots.yml
```

# DOCX to Markdown

Now let's save the DOCX file as a Markdown file.

```bash pause=2
stencila convert examples/output/sunspots.docx examples/output/sunspots.md
```

View the Markdown file:

```bash pause=3
cat examples/output/sunspots.md
```

# DOCX back to Jupyter Notebook

After saving changes to the DOCX file, let's convert it back to a Jupyter Notebook.

```bash pause=2
stencila convert examples/output/sunspots.docx examples/output/sunspots-updated.ipynb
```

And open the notebook:

```bash pause=5
jupyter notebook examples/output/sunspots-updated.ipynb &
```

# Serving project folders

Serving a project folder allows you to view files in various formats
locally using a web browser.

In this demo, we're using `&` to run `stencila serve --sync` in the background.
You can just use `stencila serve [folder] --sync`.

The `--sync` option lets you view changes in the browser automatically on file save.

```bash pause=5
stencila serve examples --sync &
```

We're now serving examples at https://localhost:3001

Without having to explictly convert files, you can view various formats
as HTML pages.

Let's open the Jupyter Notebook as an HTML file (sunspots-updated.ipynb).

```bash pause=3
open http://localhost:3001/output/sunspots-updated.ipynb
```

As you make changes and save your Jupyter Notebook, you'll see the HTML output automatically update in the browser.

Thanks for watching!

```bash hidden
lsof -ti:3000 | xargs kill
lsof -ti:8888 | xargs kill
lsof -ti:8889 | xargs kill
```
