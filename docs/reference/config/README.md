# Config `struct`

## `build`: `Build`

Options for building a project or directory

# Build `struct`

## `site`: `Option<Site>`

Options for building a web site

Set to `null` to not build a site.

# Site `struct`

Configuration options for the site

## `pages`: `Option<Vec<Page>>`

The pages to generate for the current directory

Use this option to...

Defaults to using all files in a directory in file/directory name order.
Override this by providing a list of pages (and optionally their labels etc)
in the desired order.

## `index`: `Option<Vec<String>>`

A list of file name patterns to use as the "index" page for the current directory

Patterns use `glob` syntax and are case insensitive.
Defaults to `["index.*", "main.*", "readme.*"]`.
Set to `null` to not generate an `index.html` file for a directory.

## `include`: `Option<Vec<String>>`

A list of file/directory name patterns to include

Ignored if `pages` is not empty

## `exclude`: `Option<Vec<String>>`

A list of file/directory name patterns to exclude

Ignored if `pages` is not empty

## `label_case`: `Option<Case>`

The casing to use to transform file and directory names into breadcrumb labels

Set to `null` to not perform any case transformation.

## `breadcrumbs`: `Option<Breadcrumbs>`

Options for site navigation breadcrumbs

By default, breadcrumbs are shown in the toolbar of each page except those in the
root directory of the site.
Set to `null` to never show breadcrumbs.

## `images`: `Option<Images>`

Options for site image optimizations

By default, images in documents, including images that are the output of code
chunks are optimized.
Set to `null` to never perform image optimization.

# Page `struct`

## `source`: `PathBuf`

## `slug`: `Option<String>`

## `label`: `Option<String>`

# Case `enum`

Alternative case transformations

| Variant          | Description                                      |
| ---------------- | ------------------------------------------------ |
| `Camel`          | Camel case e.g. `camelCase`                      |
| `Kebab`          | Kebab case e.g. `kebab-case`                     |
| `Pascal`         | Pascal case e.g. `PascalCase`                    |
| `ScreamingSnake` | Screaming snake case e.g. `SCREAMING_SNAKE_CASE` |
| `Sentence`       | Sentence case e.g. `Sentence case`               |
| `Snake`          | Snake case e.g. `snake_case`                     |
| `Title`          | Title case e.g. `Title Case`                     |
| `Train`          | Train case e.g. `Train-Case`                     |

# Breadcrumbs `struct`

Configuration options for site navigation breadcrumbs shown in the toolbar of each page

## `root_label`: `Option<String>`

The label for the first breadcrumb pointing to the root directory of the site

Set to `null` for no label.

## `root_icon`: `Option<String>`

The icon for the first breadcrumb pointing to the root directory of the site

Set to `null` for no icon.

## `separator`: `String`

The separator between each breadcrumb

Usually a character such as `/` but may be an emoji or
a multi-character string.

# Images `struct`

## `sizes`: `Option<Vec<u32>>`

A list of image widths to generate for each of the formats

Set to `null` not only generate images of the same size as
the original.

## `quality`: `u8`

The quality of optimized images

An integer between 1 and 100 where 100 is the best quality and
thus has the largest file size.

## `formats`: `Option<Vec<ImageFormat>>`

A list of formats to generate images for

Set to `null` to only generate images in the original format
of the source image.

Note: support for AVIF is alpha and may not be enabled
in the version of Stencila you are using.

# ImageFormat `enum`

Alternative image formats

| Variant | Description |
| ------- | ----------- |
| `Avif`  |             |
| `WebP`  |             |
