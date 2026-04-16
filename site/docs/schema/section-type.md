---
title: Section Type
description: A category of section.
---

This is an enumeration used in Stencila Schema for section roles.

It includes many section types recommended by JATS, with additional values for
other sections commonly found in scholarly and technical documents.

See [`Section.sectionType`](./section.md#sectiontype) for the property that
uses this enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `SectionType`:

- [JATS sec-type values](https://jats.nlm.nih.gov/): Close analogue because many Stencila section types align with JATS-recommended section labels, with additional values for broader scholarly and technical writing workflows.

# Members

The `SectionType` type has these members:

| Member                   | Description                                                                                                                                                                                                     |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Abstract`               | A concise summary of the article’s purpose, methods, key results, and conclusions.                                                                                                                              |
| `Summary`                | A short, often technical summary such as an author or executive summary.                                                                                                                                        |
| `NonTechnicalSummary`    | Non-technical summary written for general audiences, explaining the research’s significance and real-world implications without specialized terminology.                                                        |
| `Highlights`             | Bullet-point summary of key findings or contributions (also called Key Points in some journals).                                                                                                                |
| `Introduction`           | Establishes context, prior work, and the research question or objectives.                                                                                                                                       |
| `Background`             | Extended context and theoretical foundation, providing deeper background than typically found in the Introduction.                                                                                              |
| `RelatedWork`            | Survey or comparison of closely related prior work (common in CS/ML).                                                                                                                                           |
| `Materials`              | Details on materials, datasets, organisms, hardware, or reagents used.                                                                                                                                          |
| `Methods`                | Procedures, algorithms, and analysis methods sufficient for reproducibility.                                                                                                                                    |
| `ExperimentalDesign`     | Detailed description of experimental design, including apparatus, parameters, conditions, and protocols.                                                                                                        |
| `StatisticalAnalysis`    | Detailed description of statistical methods, including model specifications, power/sample-size calculations, and analysis decisions.                                                                            |
| `Cases`                  | Case reports or case-series descriptions, often in clinical research.                                                                                                                                           |
| `Results`                | Objective presentation of findings without extensive interpretation.                                                                                                                                            |
| `Discussion`             | Interpretation of results, implications, and relation to prior work.                                                                                                                                            |
| `Limitations`            | Known constraints or threats to validity affecting interpretation or generalizability.                                                                                                                          |
| `Conclusions`            | Final takeaways, recommendations, and wrap-up of the study’s contribution.                                                                                                                                      |
| `FutureWork`             | Suggested directions, next steps, or open problems for subsequent research.                                                                                                                                     |
| `References`             | Bibliographic list of works cited in the document.                                                                                                                                                              |
| `Acknowledgements`       | Recognition of non-author contributions such as assistance, facilities, or feedback.                                                                                                                            |
| `Declarations`           | General declarations section that may contain or encompass various types of formal statements required by journals, including funding, competing interests, ethics, consent, and other compliance declarations. |
| `Funding`                | Sources of financial support, grant numbers, and funding acknowledgments.                                                                                                                                       |
| `CompetingInterests`     | Declarations of conflicts or competing interests by the authors.                                                                                                                                                |
| `Ethics`                 | Ethical approvals, IRB/IEC statements, animal welfare, or ethical compliance.                                                                                                                                   |
| `ConsentStatements`      | Statements confirming informed consent was obtained from participants, patients, or for publication of identifying information.                                                                                 |
| `AuthorContributions`    | Specific roles and contributions of each author (e.g., CRediT taxonomy).                                                                                                                                        |
| `DataAvailability`       | Statement describing where and how the underlying data can be accessed.                                                                                                                                         |
| `CodeAvailability`       | Statement describing where and how to access analysis code, software, or computational notebooks used in the research.                                                                                          |
| `Reproducibility`        | Comprehensive statement on reproducibility and openness, covering availability of data, code, materials, and protocols.                                                                                         |
| `Abbreviations`          | List of abbreviations used in the document and their definitions.                                                                                                                                               |
| `Nomenclature`           | Glossary, symbols, or notation used throughout the document.                                                                                                                                                    |
| `Preregistration`        | Statement or link to study preregistration, including hypotheses and analysis plans registered before data collection.                                                                                          |
| `SupplementaryMaterials` | Additional figures, tables, data, or methods that support the main text.                                                                                                                                        |
| `Appendix`               | Ancillary material such as derivations, proofs, or extended details.                                                                                                                                            |
| `Main`                   | The main body of content when a document is not subdivided into standard sections.                                                                                                                              |
| `Header`                 | Front matter at the top of a page or section (e.g., running titles).                                                                                                                                            |
| `Footer`                 | Foot matter at the bottom of a page or section (e.g., footers, disclaimers).                                                                                                                                    |
| `Iteration`              | A section representing an iteration of a `ForBlock`.                                                                                                                                                            |

# Bindings

The `SectionType` type is represented in:

- [JSON-LD](https://stencila.org/SectionType.jsonld)
- [JSON Schema](https://stencila.org/SectionType.schema.json)
- Python type [`SectionType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`SectionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/section_type.rs)
- TypeScript type [`SectionType`](https://github.com/stencila/stencila/blob/main/ts/src/types/SectionType.ts)

***

This documentation was generated from [`SectionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/SectionType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
