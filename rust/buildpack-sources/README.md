# 📦 Stencila buildpack for project sources

Detects `project.json` files and imports any `sources` defined within it (e.g. GitHub repos, Google Drive files).

This buildpack is unusual in that it modifies the working directory during the `detect` phase of the buildpack lifecycle (and does not add a layer during the `build` phase). This is necessary given its purpose: to import sources in to the working directory so that other buildpacks can run their own `detect` phase over those sources.
