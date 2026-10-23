PMC bucket fixtures

Recorded from https://pmc-oa-opendata.s3.amazonaws.com/:
- PMC5697743.versions.xml: ListObjectsV2 with prefix=PMC5697743. and delimiter=/,
  recorded 2026-10-23.
- PMC5697743.objects.xml and PMC11193654.objects.xml: ListObjectsV2 with
  prefix=PMCID. and max-keys=1000, recorded 2026-10-22. The tests reuse the
  object entries for the selected-prefix listing (the server's Prefix echo
  does not affect parsing).
- PMC5697743.1.json and PMC9524584.1.json: unmodified article manifests,
  recorded 2026-10-22.
- emss-73944-f001.jpg: original figure from PMC5697743.1, with the MD5 recorded
  in the real manifest. It is used without resizing or conversion.

The 2026-10-22 recordings came from the HHMI corpus fetcher's HTTP cache and
papers/*/sources/pmc fixtures. These tests use only the copies in this folder.

article.xml is a reduced JATS fixture using the first figure's extensionless
reference and a tiny CSV supplement (data.csv). article.json is derived from
PMC5697743.1.json: it lists only those two assets, has the reduced XML/CSV MD5s,
and sets is_retracted=true to exercise metadata. Tests also derive a manuscript
with is_pmc_openaccess=false and absent media_urls, and synthetic paginated
listings with versions 9, 10 and 319 and XML-escaped continuation tokens.

Decoded manifest metadata is stored in Article.licenses and the existing
Article.extra map as pmc.is_retracted, pmc.is_manuscript and
pmc.is_pmc_openaccess. extra is flattened during serialization.

The retired oa.fcgi/FTP downloader has been removed. Issue
hhmi-debranded-pdfs-2026/issues/2-pmc-tar-url-404.md is superseded by this
migration; no deprecated/ URL patch is needed. Legacy .tar.gz decoding remains
for existing cached packages and local archives, including media retention.
