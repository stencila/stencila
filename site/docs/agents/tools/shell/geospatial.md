---
title: "Geospatial"
description: "Guards against destructive operations on NetCDF/HDF climate data files. Guards against destructive GDAL/OGR geospatial data operations"
---

This page lists the safe and destructive patterns in the **Climate Data Tools** and **GDAL/OGR** shell guard packs. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## Climate Data Tools

**Pack ID:** `geospatial.climate_data`

Guards against destructive operations on NetCDF/HDF climate data files

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `geospatial.climate_data.ncdump` | `^ncdump\b[^\|><]*$` |
| `geospatial.climate_data.cdo_info` | `^cdo\s+(?:info\|sinfo\|showname\|showvar)\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `geospatial.climate_data.ncatted_overwrite` | Overwrites the input NetCDF file when editing attributes | Use `-o output.nc` to write to a new file instead of overwriting | Medium |
| `geospatial.climate_data.ncks_overwrite` | Overwrites existing NetCDF output files | Remove `-O`; use a new output filename | Medium |
| `geospatial.climate_data.cdo_replace` | CDO overwrite mode replaces existing output files | Remove `-O`; use a new output filename or backup first | Medium |

## GDAL/OGR

**Pack ID:** `geospatial.gdal`

Guards against destructive GDAL/OGR geospatial data operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `geospatial.gdal.gdalinfo` | `^gdalinfo\b[^\|><]*$` |
| `geospatial.gdal.ogrinfo` | `^ogrinfo\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `geospatial.gdal.gdal_translate_overwrite` | Overwrites existing raster output files without confirmation | Remove `--overwrite`; rename the output file or backup first | Medium |
| `geospatial.gdal.gdalwarp_overwrite` | Overwrites existing warped raster output | Remove the overwrite flag; use a new output filename | Medium |
| `geospatial.gdal.ogr2ogr_overwrite` | Overwrites existing vector dataset output | Remove the overwrite flag; use a new output filename | Medium |
| `geospatial.gdal.ogr2ogr_update_delete` | Modifies existing vector datasets by appending or updating features | Backup the target dataset first; review with `ogrinfo` | Medium |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/geospatial.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/geospatial.rs).
