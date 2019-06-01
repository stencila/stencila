# Demos and tutorials

> :wrench: Make a list of demos here with embedded asciinema.org links

## Contributing

We :heart: new demos or tutorials!

### Requirements

You should fist compile this project so that the CLI is available as a Node.js script:

```
npm install
npm run build:dist
```

This directory contains a symbolic link called `stencila` which points to the compiled script. This is available from within the demos. You can check it is working like this:

```bash
./stencila --help
```

If you want to record and publish a demo then you'll also need to install the awesome [Asciinema](https://asciinema.org/) :movie_camera:. See the [installation instructions](https://asciinema.org/docs/installation) for your platform.

### Writing your demo

Write your demo or tutorial in a Markdown file e.g. `tutorial-convert.md`.
Headings and paragraph, and most inline elements, are supported. These will be encoded as comments in the Bash demo.

Use a code block with `bash` as the language to execute a command e.g.

````md
```bash
stencila --help
```
````

If you want to pause the script to after a command, add the `pause` directive with the number of seconds to pause e.g.

````md
```bash pause=2
stencila --help
```
````

### Running your demo

To run your demo interactively e.g. for a tutorial use `make run-<name-of-demo-without-extension>`. e.g.

```bash
make run-tutorial-convert
```

This mode of running the demo requires that you press enter when you want the demo to progress.

To preview your demo before recording a screencast, without needing to press enter for each line, use `make preview-<name-of-demo-without-extension>` e.g.

```bash
make preview-tutorial-convert
```

### Recording your demo

You can create a screencast of your demo using use `make record-<name-of-demo-without-extension>` e.g.

```bash
make record-tutorial-convert
```

Use `make play-<name-of-demo-without-extension>` to play your recorded screencast and `make upload-<name-of-demo-without-extension>` to upload it to https://asciinema.org.
