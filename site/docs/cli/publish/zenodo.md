---
title: "`stencila publish zenodo`"
description: Publish to Zenodo
---

Publish to Zenodo

# Usage

```sh
stencila publish zenodo [OPTIONS] [PATH]
```

# Examples

```bash
Further information

Authentication

To deposit a document at Zenodo, you must first have an authentication token that has the deposit:actions scope enabled.

To create an authentication token, log into Zenodo, visit your account's dashboard, then click Applications, followed by + New Token within the Personal access tokens  section. Give the token a name and enable the deposit:actions the scope. Enable the deposit:write scope to enable the --force flag.

To inform Stencila about the new token, add it as the STENCILA_ZENODO_TOKEN environment variable or include it as the --token <TOKEN> option.

Recommended workflow

We recommend starting with the Zenodo Sandbox at <https://sandbox.zenodo.org/>.

  $ export STENCILA_ZENODO_TOKEN="<TOKEN>" # from https://sandbox.zenodo.org/
  $ stencila publish zenodo <DOCUMENT_PATH>
  🎉 Draft deposition submitted
  🌐 URL: https://sandbox.zenodo.org/deposit/<deposit-id> (visit to check details and publish)
  📑 DOI: 10.5282/zenodo.<deposit-id>
  Note: This deposit has been submitted to the Zenodo Sandbox.
  Note: Use the --zenodo flag to resubmit to the production Zenodo server.

You should now review the deposit, make any corrections and then click publish from Zenodo's web interface when you're happy. If you wish to skip the review process and publish immediately, then use the --force flag.

Now that you have an understanding of the process, you should move to the Zenodo production server at <https://zenodo.org/>. This involves creating an authentication token there, informing Stencila about it and then adding the --zenodo flag as a command-line argument.

  $ export STENCILA_ZENODO_TOKEN="<TOKEN>" # from https://zenodo.org/
  $ stencila publish zenodo --zenodo <DOCUMENT_PATH>
  🎉 Draft deposition submitted
  🌐 URL: https://zenodo.org/deposit/<deposit-id> (visit to check details and publish)
  📑 DOI: 10.5281/zenodo.<deposit-id>

Metadata

Metadata for the deposition is provided by command-line arguments, falling back to metadata found within the document, then Stencila's defaults.

Zenodo requires that deposits have metadata such as title and description. It also requires that you describe which resource type and/or publication type the deposit is.

By default, Stencila describes your document as a publication, with the preprint sub-type. You can use the --lesson flag to describe your document as a lesson. To use another publication sub-type, review the list in the documentation above and provide it as the --publication=[<PUBLICATION_TYPE>] option.

Every source format has its own mechanism for providing metadata. For example, within Stencila Markdown (.smd files), you add YAML front matter:

---
title: Example Stencila Markdown
description: An example of a Stencila Markdown document with embedded metadata
---
```

# Arguments

| Name     | Description                                              |
| -------- | -------------------------------------------------------- |
| `[PATH]` | Path to location of what to publish. Default value: `.`. |

# Options

| Name                  | Description                                                                                                     |
| --------------------- | --------------------------------------------------------------------------------------------------------------- |
| `--token`             | Zenodo authentication token.                                                                                    |
| `--sandbox`           | Publish to the Zenodo Sandbox for testing. Possible values: `true`, `false`. Default value: `true`.             |
| `--zenodo`            | Specify Zenodo instance, defaults to the public-facing production server. Default value: `zenodo.org`.          |
| `--lesson`            | Upload document as a "lesson". Possible values: `true`, `false`.                                                |
| `--reserve-doi`       | Reserve a DOI for the deposition (overrides DOI in Article metadata, if any). Possible values: `true`, `false`. |
| `--doi`               | Supply an existing DOI.                                                                                         |
| `--publication-date`  | Publication date.                                                                                               |
| `--title`             | Title to use for the deposit.                                                                                   |
| `--description`       | Description notes (HTML permitted).                                                                             |
| `--license`           | License Identifier (examples: cc-by, cc0).                                                                      |
| `--closed`            | Closed Access. Possible values: `true`, `false`.                                                                |
| `--restricted`        | Set `--access-right` to restricted. Possible values: `true`, `false`.                                           |
| `--embargoed`         | Provide a date when the embargo ends.                                                                           |
| `--access-conditions` | Conditions to fulfill to access deposition (HTML permitted).                                                    |
| `--access-right`      | Access right. Default value: `open`.                                                                            |
| `--keywords`          | Comma-delimited list of keywords.                                                                               |
| `--method`            | Methodology (HTML permitted).                                                                                   |
| `--notes`             | Additional notes (HTML permitted).                                                                              |
| `--version`           | Version of document.                                                                                            |
| `--publication`       | Upload document as a "publication". Default value: `preprint`.                                                  |
| `--force`             | Publish the deposition immediately (use with care. Possible values: `true`, `false`.                            |
| `--dry-run`           | Dry run mode - no actual upload. Possible values: `true`, `false`.                                              |

**Possible values of `--publication`**

| Value                    | Description |
| ------------------------ | ----------- |
| `annotation-collection`  |             |
| `book`                   |             |
| `section`                |             |
| `conference-paper`       |             |
| `data-management-plan`   |             |
| `article`                |             |
| `patent`                 |             |
| `preprint`               |             |
| `deliverable`            |             |
| `milestone`              |             |
| `proposal`               |             |
| `report`                 |             |
| `software-documentation` |             |
| `taxonomic-treatment`    |             |
| `technical-note`         |             |
| `thesis`                 |             |
| `working-paper`          |             |
| `other`                  |             |

**Possible values of `--access-right`**

| Value        | Description                                                                        |
| ------------ | ---------------------------------------------------------------------------------- |
| `open`       | Open Access. Sets the default license to CC-BY, e.g. --license='cc-by'.            |
| `embargoed`  | Embargoed Access. Requires --access_conditions, --license, and --embargoed=<DATE>. |
| `restricted` | Restricted Access. Requires --access_conditions.                                   |
| `closed`     | Closed Access.                                                                     |
