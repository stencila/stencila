# Loose JATS media fixtures

`PMC5754353.1` is a reduced excerpt of *Sleeping sickness is a circadian disorder*,
Rijo-Ferreira et al., Nature Communications 9, 62 (2018),
https://doi.org/10.1038/s41467-017-02484-2, licensed CC BY 4.0.
Source: https://pmc-oa-opendata.s3.amazonaws.com/PMC5754353.1/PMC5754353.1.xml
and the sibling `.json` licensing manifest (`license_code: "CC BY"`).
Retrieved anonymously on 2026-10-23. Only the title, identifiers, permissions,
two figures, and supplements are retained. Images and PDFs are small generated
stand-ins; they are not scientific data or representations of the article's
figures. Filenames and hrefs retain the actual flat bucket layout.

`nih-manuscript` reproduces the missing-file layout and filenames observed in
PMC9524584.1 (NIHMS1836442), fetched anonymously from the same bucket. That
article's manifest says `is_manuscript: true`, `is_pmc_openaccess: false`, and
`license_code: "TDM"`; its prefix has only XML, text, and JSON. No restricted
article prose, captions, images, or XML excerpt is included: the fixture is
synthetic XML using factual filenames and element structure only.

`extensionless` is wholly synthetic and exercises graphics, inline graphics,
audio, video, direct and child supplement hrefs, remote URLs, and missing files.
The 1601-pixel-wide TIFF/PNG/JPEG variants and small GIF test original resolution
and deterministic selection. The audio, video, and ZIP bytes are stand-ins;
tests do not attempt to decode them.

`../../../codec-meca/tests/fixtures/originals.meca` is a deterministic synthetic
ZIP containing JATS with an extensionless figure href, the same TIFF, and CSV.
All synthetic fixtures are available under the repository's Apache-2.0 license.
