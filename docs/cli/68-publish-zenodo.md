---
title: Publish to Zenodo
description: Using the Stencila CLI to publish documents to Zenodo
config:
  publish:
    ghost:
      slug: cli-publish-zenodo
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

```sh
stencila publish zenodo
```

# Publishing to Zenodo from the Command Line

The `stencila publish` command has multiple "publishers" that allow Stencila users to be able to publish documents to various platforms. Zenodo is a digital library where researchers can share their work and get a digital object identifier or [DOI](https://en.wikipedia.org/wiki/Digital_object_identifier). Using Zenodo you can make your publications, reports, documents and pre-prints available and more FAIR (Findable, Accessible, Interoperable and Reusable). All scientific disciplines are welcome to archive documents in Zenodo. Having a persistent identifier (DOI) allows others to cite the authoritative copy and a specific version of your document. This will ensure the document can be archived and available for a very long time, even as projects, PIs and institutions come and go.

This tutorial will show you how to take a Stencila document and push it to Zenodo. To push to Zenodo you'll need to [sign up for an account](https://help.zenodo.org/docs/get-started/create-an-account/). 

## Learning objectives

By the end of this tutorial you should be able to:

- Understand how Stencila Documents are uploaded to Zenodo with appropriate metadata
- Understand Stencila Metadata and how it maps to required Zenodo metadata
- Learn how to create an API key for Zenodo
- Test publish a document to the Zenodo Sandbox server

## Stencila and Zenodo 

The `publish zenodo` sub command of the Stencila CLI can help you to publish documents to Zenodo for long-term archival and registerting a persistent identifier or DOI. This can be useful when you have a report, findings, pre-print, or other manuscript which you would like to share with others. Zenodo will keep a copy of this document. Zenodo can support other non-document data types, such as research datasets. We'll only be dealing with documents in this tutorial. The [Zenodo documentation](https://help.zenodo.org/docs/) is a great reference for learning about Zenodo in general and how ti supports other deposition types as well. 

Stencila documents are converted to the [Stencila Schema](/docs/schemaFIXME), a rich and descriptive Schema which describes different parts of a document. As such, there are various pieces of metadata that can be drawn upon directly to craft a Zenodo metadata record, and a few other variables required by Zenodo which can also be added as ad hoc metadata in your Stencila Document. This can streamline the process of uploading a document to Zenodo. 

In this way you can create Stencila documents which are able to be pushed to your Zenodo in a way that they're largely self-describing as to things like `title`, `description`, `authors` etc. This tutorial is itself a Stencila document being pushed to Zenodo. If you're curious you can see the source at [here](https://github.com/stencila/stencila/main/docs/tutorials/publish-to-zenodo/tutorial.md).

The `stencila publish zenodo` cli command uses the Zendo API to publish your document into Zenodo. We do this by default using the `sandbox.zenodo.org` Zenodo endpoint. **This is a non-production endpoint for Zenodo that you can use to test Zenodo publishing before you submit to the production system. We recommend that you use the sandbox server first as you get to know this command.** 

A basic form of the `publish zenodo` command is:

```
stencila publish zenodo somefile.smd --token XXXXXX 
```

Lets break that down:

1. Run the `stencila` CLI
2. Run the `publish` subcommand
3. Run the `zenodo` variant of publish
4. Use the contents of `somefile.smd` as the page content
5. `-token` is used to show you have permissions (can also be provided by setting STENCILA_ZENODO_TOKEN environment variable)

With this command the contents of the document will be pushed to the Ghost server.  Note that we've included all the required metadata that Zenodo requires for creating a Zenodo record in the document metadata. In this way we don't have to use all of the command line flags to specify data that can be held in the document. 

If you run `stencila publish zenodo -h` you can learn about other features and flags:

```
Publish to Zenodo

Usage: stencila publish zenodo [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to location of what to publish [default: .]

Options:
      --debug    Display debug level logging and detailed error reports
      --force    Publish the deposition immediately (use with care
      --dry-run  Dry run mode - no actual upload
  -h, --help     Print help (see more with '--help')

Zenodo Settings:
      --sandbox            Publish to the Zenodo Sandbox for testing
      --token <TOKEN>      Zenodo authentication token [env: STENCILA_ZENODO_TOKEN]
      --zenodo[=<ZENODO>]  Specify Zenodo instance, defaults to the public-facing production server [default: zenodo.org]

Deposition Settings:
      --doi <DOI>                         Supply an existing DOI
      --lesson                            Upload document as a "lesson"
      --publication[=<PUBLICATION_TYPE>]  Upload document as a "publication" [default: preprint] [possible values: annotation-collection, book, section, conference-paper, data-management-plan,
                                          article, patent, preprint, deliverable, milestone, proposal, report, software-documentation, taxonomic-treatment, technical-note, thesis, working-paper,
                                          other]
      --reserve-doi                       Reserve a DOI for the deposition (overrides DOI in Article metadata, if any)

Deposition Metadata:
      --access-conditions <ACCESS_CONDITIONS>  Conditions to fulfill to access deposition (HTML permitted)
      --access-right <ACCESS_RIGHT>            Access right [default: open] [possible values: open, embargoed, restricted, closed]
      --closed                                 Closed Access
      --description <DESCRIPTION>              Description notes (HTML permitted)
      --embargoed <YYYY-MM-DD>                 Provide a date when the embargo ends
      --keywords=<KEYWORDS>                    Comma-delimited list of keywords
      --license <LICENSE>                      License Identifier (examples: cc-by, cc0)
      --method <METHOD>                        Methodology (HTML permitted)
      --notes <NOTES>                          Additional notes (HTML permitted)
      --publication-date <YYYY-MM-DD>          Publication date
      --restricted                             Set `--access-right` to restricted
      --title <TITLE>                          Title to use for the deposit
      --version <VERSION>                      Version of document

```

To see a longer form of this with some examples, run `stencila publish zenodo --help`

That's a basic overview of how it works, lets dive into a bit more of a detailed example where we look at the various metdata fields that can be supported and a few more advanced features of the `publish zenodo` Stencila subcommand.

## Concepts

The following concepts are useful to know about as you begin publishing Stencila documents to Ghost.

- Stencila CLI
- Zenodo Deposition Metadata
- API Keys

### CLI

 The `stencila` command is referred to as the Stencila CLI, or command line interface. It is a command line tool which acts as the interface to Stencila for users and integrated systems. You can learn more about all the features of the CLI at the [CLI Reference page](https://FIXME). Stencila is written in the programming language Rust, and has many features that can be useful for developing runable and reproducible documents with a mix of prose and code.

### Zenodo Deposition Metadata

Deposition metadata in Zenodo must have a few fields filled out at a minimum:


| Attribute                               | Required                                  |
| --------------------------------------- | ----------------------------------------- |
| `upload_type`              | Yes                                       |
| `publication_type`          | Yes, if `upload_type` is `"publication"`. |
| `publication_date`          | Yes                                       |
| `title`                     | Yes                                       |
| `creators`        | Yes                                       |
| `description` | Yes                                       |
| `access_right`              | Yes                                       |

The full schema for Zenodo is [available here](https://developers.zenodo.org/#entities)


### API

The CLI tool allows for publishing to the [Zenodo Sandbox](https://sandbox.zenodo.org/), or the Zenodo production servers. The Sandbox is a great way to practice and get to know how the publish tool works without the risk of poluting the global DOI space, or the production Zenodo server. So we reccommend that you use it to begin with. Each server, the sandbox or production server have their own account and API keys, so to get started you should create an account on each server. Tokens can be created in My Account -> Applications [Sandbox](https://sandbox.zenodo.org/account/settings/applications/tokens/new/) & [Production](https://zenodo.org/account/settings/applications/tokens/new/). Publishing to Zenodo is a two-step process, and the API permission `deposit:actions` allows you to write metadata but not publish, `deposit:write` allows you to publish. Depending on your needs, set the API key to the minimal level that can meet your needs. 

## Creating a deposit 

A Zenodo document can be any number of data types, but Stencila documents are usually publications or articles, so we've restricted the `stencila publish zenodo` command to working with articles. We try to read most metadata like creators (reading `authors:`), title, description from the document itself if set. So in our examples we'll be adding that metadata to the document yaml headers. Note that the CLI also supports overriding this metadata with command line flags. If you need to tweak or change the title, description etc, you can do so by supplying flags to the commandline utility at runtime. 

As we saw in the above example, you'll need:

1. To decide if you'll be using the sandbox or production server
2. To create an API token on the [Sandbox](https://sandbox.zenodo.org/account/settings/applications/tokens/new/) & [Production](https://zenodo.org/account/settings/applications/tokens/new/) servers.
3. Have a source document in some flavor of Markdown (or other supported format)
4. The [Stencila CLI](https://github.com/stencila/stencila/releases) installed


Say your API key is something like:
`XvIQ6SQ27sqACdGPlkzuNy6SFtML34r608VNuedU0NT0cacYB3CUNixO5Uzv`

You'd now be able to run:

```
export STENCILA_ZENODO_TOKEN=XvIQ6SQ27sqACdGPlkzuNy6SFtML34r608VNuedU0NT0cacYB3CUNixO5Uzv
stencila publish ghost somefile.smd --sandbox
```

You can select `--post` or `--page` to create a post or a page in your Ghost CMS.

the contents of `somefile.smd` might be something like:

```
---
title: My research report 
description: A research report on something very interesting
access-right: open
authors: ["John Doe", "Jane Doe"]
publish: 
  zenodo:
    license: cc0
    embargoed: false
    notes: Some extra notes about a deposition
    methodology: A paragraph describing the methodology of the study. 
---

Note the `publish:` yaml key can have a `zenodo:` sub key which can be used to set many of the same values you can set with commandline flags. 

# Introduction 

This report will detail some interesting topics. 

## Methods 

We used a very good method!

## Conclusion 

The end!
```

With the above `stencila publish ghost` command, Stencila will convert the `.smd` file to Lexical, import it into Ghost and write back the ID of the file into the headers of the file. This ID, now persisted in the file, links the file to Ghost and prevents you from having to specify it at the commandline. In this way updates can be pushed to the document each time you make changes to the source text of the file.

The metadata section after you push once would look like:

```
---
title: My research report 
description: A research report on something very interesting
tags: ['report','science']
slug: research-report
identifiers:
  - https://myawesomepage.ghost.org/ghost/api/admin/posts/678efc86c03bfb0001d3f1a1/
---
```

This identifier in the `identifiers` key is what enables you to be able to re-run `stencila publish ghost` on the same file and have it update your CMS page. `description:` is mapped to the `excerpt` in Ghost and is used for [Open Graph](https://ogp.me/) metadata for the page, used for link previews and such.

With this file connected by the `identifier` value, you'll be able to make additions or changes to the file and simply re-run `stencila publish ghost somefile.smd` against it and have any changes pushed up to Ghost.

## Key Points


- Metadata values in YAML headers can be used to simplify calls to the `publish zenodo` utility. While many options can be specifed, or overridden using command-line flags, having values in the metadata can simplify workflows.
- To create an API token at Zenodo you need to create an "Application" in your Zenodo profile, with an access token, and share with the Stencila CLI your  token using either a commandline flag `--token` or environment variable `STENCILA_ZENODO_TOKEN`
- We've shown you how to push a test document up to your Ghost instance. Now you can create your own customizations and integrations to make runnable documents push to Ghost.
