---
title: Stencila Enhancement Proposals
author:
  - Nokome Bentley
type: Process
status: Draft
---

# Introduction

A Stencila Enhancement Proposal (StEP) is a proposal for a enhancement to Stencila. Each StEP is a design document for a process, feature or fix with a concise specification of it's rationale and specification. StEPs often also provide auxiliary files with guidance on implementation approaches.

StEPs are inspired by PEPs (Python Enhancement Proposals). Rather than reinvent the wheel, StEPs borrow many concepts (and name!) from PEPs and this document draws much from [PEP 1](https://www.python.org/dev/peps/pep-0001/).

# Creating a StEP

StEPs are usually created for larger features or fixes. If you have a feature idea it worthwhile to discuss it on the [community forum](https://community.stenci.la/) before submitting. For smaller bugs please use [Github issues](https://github.com/stencila/stencila/issues). If you're not sure, we're happy to chat about it on the [community forum](https://community.stenci.la/) - pop on over and say hi!

## Format

StEPs are written in Markdown. Markdown is used because of it's wide use amongst developers (and increasingly researchers), because it is rendered nicely by default on Github, and easily integrated with documentation tools.

## File name

StEPs should have a file name with the pattern `step-XXXX-SLUG.md`, where `XXXX` is the StEP number and `SLUG` is a human readable shortened title (e.g. `cell-editor`).

## Header

Each StEP should begin with a YAML header:

```yaml
---
title: <pep title>
author:
  - <first author>
  - <second author>
  - <third author>
type: <Process | Feature | Fix>
status: <Draft | Active | Accepted | Deferred | Rejected | Withdrawn | Final | Superseded>
---
```
Definitions:

**Process** - A Process StEP describes a process surrounding Stencila, or proposes a change to a process. Examples include procedures, guidelines, changes to the decision-making process, and changes to Stencila.

**Feature** - A Feature StEP describes a new feature for Stencila, or proposes a major change to an existing feature. A feature could be an entirely new or an extension to an existing feature.

**Fix** - A Fix StEP identifies a feature that is not working and proposes a solution to get up and running. Alternately a Fix StEP may propose an alternate way to solve, approach, or address an existing (working) feature.



## Parts of a great StEP
The StEP template below includes the following headings.

**Introduction**: Provide an informal context and motivation for the StEP.

**Definitions**: Provide terms used in Stencila, and define any new terms introduced in the StEP.

**Concepts**: Explain concepts and rationale for the approach.

**Distinction**: If necessary position with respect to existing solutions. For example, why a proposed Fix StEP is better than an existing solution.

**Examples**: Include examples!

**Implementation**: Links to specifications, notes on dependencies, or other conditions for implementation.

**Auxiliary files**: StEPs may include auxiliary files such as diagrams, code samples, or Markdown files. Such files must start with the filename of the StEP (e.g. `0001-cell-editor-auxiliary-file.md`).

## Submitting

Check in with the team on the [community forum](https://community.stenci.la/) as you're thinking about submitting a StEP. As with all contributions, please review our contributor guide found at CONTRIBUTING.md and our [Code of Conduct](https://github.com/stencila/stencila/blob/develop/CONDUCT.md). When you're ready to submit a StEP, send a pull request to https://github.com/stencila/stencila to add the Markdown file, (titled following the format  `step-XXXX-SLUG.md`, where `XXXX` is the StEP number and `SLUG` is a human readable shortened title (e.g. `cell-editor`)) to [this repository](https://github.com/stencila/stencila/tree/develop/docs/steps). Create another pull request to add your StEP to the [README.md](https://github.com/stencila/stencila/blob/develop/docs/steps/README.md). Let us know if you have any questions about the StEP format or submission process. 



# StEP Template
Copy the markdown below to begin writing a StEP. Thank you for your contribution!

```yaml
---
title: <pep title>
author:
  - <first author>
  - <second author>
  - <third author>
type: <Process | Feature | Fix>
status: <Draft | Active | Accepted | Deferred | Rejected | Withdrawn | Final | Superseded>
---
```
# Introduction
Provide an informal context and motivation for the StEP.
# Definitions
Provide terms used in Stencila, and define any new terms introduced in the StEP.
# Concepts
Explain concepts and rationale for the approach.
# Distinction
If necessary position with respect to existing solutions. For example, why a proposed Fix StEP is better than an existing solution.
# Examples
Include examples!
# Implementation
Links to specifications, notes on dependencies, or other conditions for implementation.
# Auxiliary files
List of related files. Please name them starting with the filename of the StEP (e.g. `0001-cell-editor-auxiliary-file.md`).
