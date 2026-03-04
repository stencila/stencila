//! Geospatial packs: `geospatial.gdal`, `geospatial.climate_data`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, safe_pattern};

// ---------------------------------------------------------------------------
// geospatial.gdal
// ---------------------------------------------------------------------------

pub static GDAL_PACK: Pack = Pack {
    id: "geospatial.gdal",
    name: "GDAL/OGR",
    description: "Guards against destructive GDAL/OGR geospatial data operations",
    safe_patterns: &[
        safe_pattern!("gdalinfo", r"^gdalinfo\b[^|><]*$"),
        safe_pattern!("ogrinfo", r"^ogrinfo\b[^|><]*$"),
    ],
    destructive_patterns: &[
        destructive_pattern!(
            "gdal_translate_overwrite",
            r"\bgdal_translate\b.*--overwrite\b",
            "Overwrites existing raster output files without confirmation",
            "Remove `--overwrite`; rename the output file or backup first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "gdalwarp_overwrite",
            r"\bgdalwarp\b.*(?:--overwrite|-overwrite)\b",
            "Overwrites existing warped raster output",
            "Remove the overwrite flag; use a new output filename",
            Confidence::Medium
        ),
        destructive_pattern!(
            "ogr2ogr_overwrite",
            r"\bogr2ogr\b.*(?:--overwrite|-overwrite)\b",
            "Overwrites existing vector dataset output",
            "Remove the overwrite flag; use a new output filename",
            Confidence::Medium
        ),
        destructive_pattern!(
            "ogr2ogr_update_delete",
            r"\bogr2ogr\b.*(?:-update|-append)\b",
            "Modifies existing vector datasets by appending or updating features",
            "Backup the target dataset first; review with `ogrinfo`",
            Confidence::Medium
        ),
    ],
};

// ---------------------------------------------------------------------------
// geospatial.climate_data
// ---------------------------------------------------------------------------

pub static CLIMATE_DATA_PACK: Pack = Pack {
    id: "geospatial.climate_data",
    name: "Climate Data Tools",
    description: "Guards against destructive operations on NetCDF/HDF climate data files",
    safe_patterns: &[
        safe_pattern!("ncdump", r"^ncdump\b[^|><]*$"),
        safe_pattern!(
            "cdo_info",
            r"^cdo\s+(?:info|sinfo|showname|showvar)\b[^|><]*$"
        ),
    ],
    destructive_patterns: &[
        destructive_pattern!(
            "ncatted_overwrite",
            r"\bncatted\b.*(?:-O|--overwrite)\b",
            "Overwrites the input NetCDF file when editing attributes",
            "Use `-o output.nc` to write to a new file instead of overwriting",
            Confidence::Medium
        ),
        destructive_pattern!(
            "ncks_overwrite",
            r"\bncks\b.*(?:-O|--overwrite)\b",
            "Overwrites existing NetCDF output files",
            "Remove `-O`; use a new output filename",
            Confidence::Medium
        ),
        destructive_pattern!(
            "cdo_replace",
            r"\bcdo\b.*(?:-O|--overwrite)\b",
            "CDO overwrite mode replaces existing output files",
            "Remove `-O`; use a new output filename or backup first",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- GDAL/OGR --

    #[test]
    fn gdal_translate_overwrite_matches() {
        let re = Regex::new(rule_by_id(&GDAL_PACK, "gdal_translate_overwrite").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("gdal_translate --overwrite input.tif output.tif"));
        assert!(re.is_match("gdal_translate -of GTiff --overwrite in.tif out.tif"));
        assert!(!re.is_match("gdal_translate input.tif output.tif"));
        assert!(!re.is_match("gdal_translate -of GTiff in.tif out.tif"));
        assert!(!re.is_match("gdalinfo input.tif"));
    }

    #[test]
    fn gdalwarp_overwrite_matches() {
        let re = Regex::new(rule_by_id(&GDAL_PACK, "gdalwarp_overwrite").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("gdalwarp --overwrite input.tif output.tif"));
        assert!(re.is_match("gdalwarp -overwrite input.tif output.tif"));
        assert!(re.is_match("gdalwarp -t_srs EPSG:4326 --overwrite in.tif out.tif"));
        assert!(!re.is_match("gdalwarp input.tif output.tif"));
        assert!(!re.is_match("gdalwarp -t_srs EPSG:4326 in.tif out.tif"));
        assert!(!re.is_match("gdalinfo input.tif"));
    }

    #[test]
    fn ogr2ogr_overwrite_matches() {
        let re = Regex::new(rule_by_id(&GDAL_PACK, "ogr2ogr_overwrite").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("ogr2ogr --overwrite out.shp in.shp"));
        assert!(re.is_match("ogr2ogr -overwrite out.shp in.shp"));
        assert!(re.is_match("ogr2ogr -f GeoJSON --overwrite out.json in.shp"));
        assert!(!re.is_match("ogr2ogr out.shp in.shp"));
        assert!(!re.is_match("ogr2ogr -f GeoJSON out.json in.shp"));
        assert!(!re.is_match("ogrinfo in.shp"));
    }

    #[test]
    fn ogr2ogr_update_delete_matches() {
        let re = Regex::new(rule_by_id(&GDAL_PACK, "ogr2ogr_update_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("ogr2ogr -update out.shp in.shp"));
        assert!(re.is_match("ogr2ogr -append out.shp in.shp"));
        assert!(re.is_match("ogr2ogr -f GeoJSON -update out.json in.shp"));
        assert!(re.is_match("ogr2ogr -f GeoJSON -append out.json in.shp"));
        assert!(!re.is_match("ogr2ogr out.shp in.shp"));
        assert!(!re.is_match("ogr2ogr -f GeoJSON out.json in.shp"));
        assert!(!re.is_match("ogrinfo in.shp"));
    }

    // -- Climate Data Tools --

    #[test]
    fn ncatted_overwrite_matches() {
        let re = Regex::new(rule_by_id(&CLIMATE_DATA_PACK, "ncatted_overwrite").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("ncatted -O -a units,temp,o,c,Kelvin input.nc"));
        assert!(re.is_match("ncatted --overwrite -a units,temp,o,c,Kelvin input.nc"));
        assert!(!re.is_match("ncatted -a units,temp,o,c,Kelvin input.nc"));
        assert!(!re.is_match("ncatted -a units,temp,o,c,Kelvin -o output.nc input.nc"));
        assert!(!re.is_match("ncdump -h input.nc"));
    }

    #[test]
    fn ncks_overwrite_matches() {
        let re = Regex::new(rule_by_id(&CLIMATE_DATA_PACK, "ncks_overwrite").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("ncks -O -v temp input.nc output.nc"));
        assert!(re.is_match("ncks --overwrite -v temp input.nc output.nc"));
        assert!(!re.is_match("ncks -v temp input.nc output.nc"));
        assert!(!re.is_match("ncks -v temp input.nc new_output.nc"));
        assert!(!re.is_match("ncdump -h input.nc"));
    }

    #[test]
    fn cdo_replace_matches() {
        let re = Regex::new(rule_by_id(&CLIMATE_DATA_PACK, "cdo_replace").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("cdo -O selname,temp input.nc output.nc"));
        assert!(re.is_match("cdo --overwrite selname,temp input.nc output.nc"));
        assert!(re.is_match("cdo -O mergetime in1.nc in2.nc out.nc"));
        assert!(!re.is_match("cdo selname,temp input.nc output.nc"));
        assert!(!re.is_match("cdo info input.nc"));
        assert!(!re.is_match("cdo sinfo input.nc"));
    }
}
