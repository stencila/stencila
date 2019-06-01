# Demos and tutorials

> :wrench: Make a list of demos here with embedded asciinema.org links

## Contributing

We :heart: new demos or tutorials!

### Requirements

We use the magical [demo-magic.sh](https://github.com/paxtonhare/demo-magic) :star2: to write reproducible demos at the command line. It's already in this folder, so there is no need to download it.

If you want to record and publish a demo then you'll need to install the awesome [Asciinema](https://asciinema.org/) :movie_camera:. See the [installation instructions](https://asciinema.org/docs/installation) for your platform.

### Getting started

Create a new Bash script with the `.sh` extension and make sure it is executable e.g.

```bash
touch tutorial-convert.sh
chmod +x tutorial-convert.sh
```

Then in the top of your new script add these two lines to enable `demo-magic` and our extensions to it:

```bash
#!/usr/bin/env bash
. demo-base.sh
```

### Writing your script

Use the `c` function to add an explanatory comment:

```bash
c "This tutorial shows you how to use the Stencila CLI for..."
```

Use the `h1`, `h2` or `h3` function to add comments (useful for longer demos and tutorials):

```bash
h1 "Getting help"
```

Use the `e` function to execute a command:

```bash
e "stencila --help"
```

If you want to pause the script to allow viewers of the recorded screencast to take things in use the `z` function:

```bash
z 2
```

### Running your script

To run your script interactively e.g. for a tutorial  use `make run-<name-of-script-without-extension>`. e.g.

```bash
make run-tutorial-convert
```

This mode of running the script requires that you press enter when you want the script to progress.

To preview your script before recording a screencast, without needing to press enter for each line, use `make preview-<name-of-script-without-extension>` e.g.

```bash
make preview-tutorial-convert
```

### Recording your script

You can create a screencast of your script using use `make record-<name-of-script-without-extension>` e.g.

```bash
make record-tutorial-convert
```

Use `make play-<name-of-script-without-extension>` to play your recorded screencast and `make upload-<name-of-script-without-extension>` to upload it to asciinema.org.

### Creating a Markdown version

You can create a Markdown version of your script (useful for giving to tutorial workshops) using `make md-<name-of-script-without-extension>` e.g.

```bash
make md-tutorial-convert
```
