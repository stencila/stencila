//! `inspect_image` tool: read-only image coordinate inspection with grid
//! overlays, rectangular crop+zoom, probe markers, and coordinate-space
//! awareness.
//!
//! Helps agents determine reliable coordinates for points and regions of
//! interest in images before authoring annotations. Returns an annotated PNG
//! and structured JSON metadata.

use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Rgba, RgbaImage};
use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::registry::{MAX_IMAGE_BYTES, ToolExecutorFn, ToolOutput};

/// Maximum total pixels (width × height) we will allocate for any intermediate
/// canvas. 64 megapixels ≈ 256 MB of RGBA data — large enough for any
/// reasonable inspection output while preventing memory exhaustion from
/// extreme pad/viewbox/zoom values.
const MAX_CANVAS_PIXELS: u64 = 64_000_000;

// ---------------------------------------------------------------------------
// Tool definition
// ---------------------------------------------------------------------------

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "inspect_image".into(),
        description: "Inspect an image to determine coordinates for features. \
            Overlays a labeled grid, zooms into crop regions, and places probe \
            crosshair markers — returning an annotated PNG and structured JSON \
            metadata with coordinates in the active coordinate space."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the source raster image (PNG, JPEG, GIF, TIFF)."
                },
                "coordinate_space": {
                    "type": "object",
                    "description": "Optional custom coordinate space. Omit for raw image pixel coordinates.",
                    "properties": {
                        "viewbox": {
                            "type": "object",
                            "description": "Maps the image into a custom coordinate system (matching Stencila SVG overlay viewBox).",
                            "properties": {
                                "x": { "type": "number" },
                                "y": { "type": "number" },
                                "width": { "type": "number" },
                                "height": { "type": "number" }
                            },
                            "required": ["x", "y", "width", "height"],
                            "additionalProperties": false
                        },
                        "pad": {
                            "type": "object",
                            "description": "Padding extending the coordinate space beyond image bounds (CSS order: top, right, bottom, left).",
                            "properties": {
                                "top": { "type": "number" },
                                "right": { "type": "number" },
                                "bottom": { "type": "number" },
                                "left": { "type": "number" }
                            },
                            "required": ["top", "right", "bottom", "left"],
                            "additionalProperties": false
                        }
                    },
                    "additionalProperties": false
                },
                "grid": {
                    "type": "object",
                    "description": "Grid overlay configuration. Provide divisions OR spacing, not both.",
                    "properties": {
                        "x_divisions": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Number of divisions along the x-axis."
                        },
                        "y_divisions": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Number of divisions along the y-axis."
                        },
                        "spacing": {
                            "type": "number",
                            "description": "Distance between grid lines in active-coordinate units."
                        },
                        "show_labels": {
                            "type": "boolean",
                            "description": "Whether to show coordinate labels. Default: true."
                        }
                    },
                    "additionalProperties": false
                },
                "crop": {
                    "type": "object",
                    "description": "Rectangular crop region in active coordinate space.",
                    "properties": {
                        "x": { "type": "number", "description": "Top-left x." },
                        "y": { "type": "number", "description": "Top-left y." },
                        "width": { "type": "number", "description": "Region width (must be > 0)." },
                        "height": { "type": "number", "description": "Region height (must be > 0)." },
                        "zoom": { "type": "number", "description": "Enlargement factor. Default: auto-fit to ~1024px." }
                    },
                    "required": ["x", "y", "width", "height"],
                    "additionalProperties": false
                },
                "probes": {
                    "type": "array",
                    "description": "Probe markers at candidate coordinates.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string", "description": "Probe identifier." },
                            "x": { "type": "number", "description": "x in active coordinate space." },
                            "y": { "type": "number", "description": "y in active coordinate space." },
                            "label": { "type": "string", "description": "Text label near the probe." },
                            "color": { "type": "string", "description": "Marker color (e.g. \"#ff0000\")." }
                        },
                        "required": ["x", "y"],
                        "additionalProperties": false
                    }
                },
                "theme": {
                    "type": "string",
                    "description": "Visual theme for grid/probe rendering.",
                    "enum": ["auto", "light", "dark"]
                },
                "sample_pixels": {
                    "type": "boolean",
                    "description": "Include sampled pixel color for each in-image probe. Default: false."
                }
            },
            "required": ["file_path"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move { execute(args, env).await })
        },
    )
}

/// Active coordinate space definition.
struct CoordSpace {
    /// Origin x of the active coordinate space.
    origin_x: f64,
    /// Origin y of the active coordinate space.
    origin_y: f64,
    /// Width of the inspectable canvas in active units.
    canvas_w: f64,
    /// Height of the inspectable canvas in active units.
    canvas_h: f64,
    /// Scale from active units to pixels (x).
    scale_x: f64,
    /// Scale from active units to pixels (y).
    scale_y: f64,
    /// Offset of the image within the active space (x).
    image_offset_x: f64,
    /// Offset of the image within the active space (y).
    image_offset_y: f64,
    /// Image width in active coordinate units.
    image_w_active: f64,
    /// Image height in active coordinate units.
    image_h_active: f64,
    /// Whether a custom viewbox/pad is in use.
    is_viewbox: bool,
    /// Padding values (if any).
    pad: Pad,
}

#[derive(Clone, Copy)]
struct Pad {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

impl Default for Pad {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

struct GridConfig {
    x_divisions: Option<u32>,
    y_divisions: Option<u32>,
    spacing: Option<f64>,
    show_labels: bool,
}

struct CropConfig {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    zoom: Option<f64>,
}

struct ProbeInput {
    id: String,
    x: f64,
    y: f64,
    label: Option<String>,
    color: Option<Rgba<u8>>,
}

/// Theme colors for drawing.
struct ThemeColors {
    line: Rgba<u8>,
    label_fg: Rgba<u8>,
    label_bg: Rgba<u8>,
}

async fn execute(
    args: Value,
    env: &dyn crate::execution::ExecutionEnvironment,
) -> AgentResult<ToolOutput> {
    let file_path = args
        .get("file_path")
        .and_then(Value::as_str)
        .ok_or_else(|| AgentError::ValidationError {
            reason: "missing required string parameter: file_path".into(),
        })?;

    let img = load_image(file_path, env).await?;
    let (img_w, img_h) = img.dimensions();

    let mut warnings: Vec<String> = Vec::new();

    let coord_space = parse_coordinate_space(&args, img_w, img_h)?;
    let grid = parse_grid(&args, &mut warnings)?;
    let crop = parse_crop(&args, &coord_space)?;
    let probes = parse_probes(&args)?;

    let theme_str = args.get("theme").and_then(Value::as_str).unwrap_or("auto");

    let sample_pixels = args
        .get("sample_pixels")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // Start with the canvas (image + padding if any).
    let mut canvas = build_canvas(&img, &coord_space)?;

    // Apply crop if requested — replaces canvas with the cropped+zoomed region.
    let effective_zoom = if let Some(ref crop_cfg) = crop {
        let zoom = apply_crop(&mut canvas, crop_cfg, &coord_space)?;
        Some(zoom)
    } else {
        None
    };

    // Determine theme colors based on the current canvas content.
    let theme = resolve_theme(theme_str, &canvas);

    // Draw grid.
    let grid_meta = grid.as_ref().map(|grid_cfg| {
        draw_grid(
            &mut canvas,
            grid_cfg,
            &coord_space,
            crop.as_ref(),
            effective_zoom,
            &theme,
        )
    });

    // Draw probes.
    let probes_meta = if !probes.is_empty() {
        Some(draw_probes(
            &mut canvas,
            &probes,
            &img,
            &coord_space,
            crop.as_ref(),
            effective_zoom,
            sample_pixels,
            &theme,
        ))
    } else {
        None
    };

    // Encode to PNG, respecting size limits.
    let png_data = encode_png(&canvas)?;

    // -- Build metadata JSON --
    let mut meta = json!({
        "dimensions": { "width": img_w, "height": img_h },
    });

    // coordinate_space
    let cs_meta = build_coord_space_meta(&coord_space);
    meta["coordinate_space"] = cs_meta;

    if let Some(gm) = grid_meta {
        meta["grid"] = gm;
    }

    if let Some(ref crop_cfg) = crop {
        meta["crop"] = json!({
            "region": {
                "x": crop_cfg.x,
                "y": crop_cfg.y,
                "width": crop_cfg.width,
                "height": crop_cfg.height,
            },
            "zoom": effective_zoom.unwrap_or(1.0),
        });
    }

    if let Some(pm) = probes_meta {
        meta["probes"] = pm;
    }

    if !warnings.is_empty() {
        meta["warnings"] = json!(warnings);
    }

    let json_text = serde_json::to_string_pretty(&meta).map_err(|e| AgentError::Io {
        message: format!("failed to serialize metadata: {e}"),
    })?;

    Ok(ToolOutput::ImageWithText {
        text: json_text,
        data: png_data,
        media_type: "image/png".into(),
    })
}

async fn load_image(
    file_path: &str,
    env: &dyn crate::execution::ExecutionEnvironment,
) -> AgentResult<DynamicImage> {
    // Check extension for supported formats.
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "tiff" | "tif" => {}
        _ => {
            return Err(AgentError::ValidationError {
                reason: format!("unsupported or unreadable image format: {file_path}"),
            });
        }
    }

    // Load through ExecutionEnvironment to honor scoping and path restrictions.
    let content = env.read_file(file_path, None, None).await?;
    match content {
        crate::execution::FileContent::Image { data, .. } => {
            // Check decoded dimensions before full decode to prevent memory
            // exhaustion from very large images.
            let reader = ImageReader::new(Cursor::new(&data))
                .with_guessed_format()
                .map_err(|_| AgentError::ValidationError {
                    reason: format!("unsupported or unreadable image format: {file_path}"),
                })?;
            if let Ok((w, h)) = reader.into_dimensions() {
                let total = u64::from(w) * u64::from(h);
                if total > MAX_CANVAS_PIXELS {
                    return Err(AgentError::ValidationError {
                        reason: format!(
                            "source image is too large ({w}×{h} = {total} pixels, \
                             max {MAX_CANVAS_PIXELS}): {file_path}"
                        ),
                    });
                }
            }
            image::load_from_memory(&data).map_err(|_| AgentError::ValidationError {
                reason: format!("unsupported or unreadable image format: {file_path}"),
            })
        }
        crate::execution::FileContent::Text(_) => Err(AgentError::ValidationError {
            reason: format!("unsupported or unreadable image format: {file_path}"),
        }),
    }
}

fn parse_coordinate_space(args: &Value, img_w: u32, img_h: u32) -> AgentResult<CoordSpace> {
    let cs = args.get("coordinate_space");

    let viewbox = cs.and_then(|v| v.get("viewbox"));
    let pad_val = cs.and_then(|v| v.get("pad"));

    let pad = if let Some(p) = pad_val {
        let top = p.get("top").and_then(Value::as_f64).unwrap_or(0.0);
        let right = p.get("right").and_then(Value::as_f64).unwrap_or(0.0);
        let bottom = p.get("bottom").and_then(Value::as_f64).unwrap_or(0.0);
        let left = p.get("left").and_then(Value::as_f64).unwrap_or(0.0);

        if top < 0.0 || right < 0.0 || bottom < 0.0 || left < 0.0 {
            return Err(AgentError::ValidationError {
                reason: "pad values must be non-negative".into(),
            });
        }

        Pad {
            top,
            right,
            bottom,
            left,
        }
    } else {
        Pad::default()
    };

    let iw = f64::from(img_w);
    let ih = f64::from(img_h);

    if let Some(vb) = viewbox {
        // Case 3: explicit viewbox.
        let vx = vb.get("x").and_then(Value::as_f64).unwrap_or(0.0);
        let vy = vb.get("y").and_then(Value::as_f64).unwrap_or(0.0);
        let vw = vb.get("width").and_then(Value::as_f64).unwrap_or(0.0);
        let vh = vb.get("height").and_then(Value::as_f64).unwrap_or(0.0);

        if vw <= 0.0 || vh <= 0.0 {
            return Err(AgentError::ValidationError {
                reason: "viewbox width and height must be positive".into(),
            });
        }

        if pad.left + pad.right >= vw || pad.top + pad.bottom >= vh {
            return Err(AgentError::ValidationError {
                reason: "pad values leave no room for the image within the viewbox".into(),
            });
        }

        let image_w_active = vw - pad.left - pad.right;
        let image_h_active = vh - pad.top - pad.bottom;
        let scale_x = iw / image_w_active;
        let scale_y = ih / image_h_active;

        Ok(CoordSpace {
            origin_x: vx,
            origin_y: vy,
            canvas_w: vw,
            canvas_h: vh,
            scale_x,
            scale_y,
            image_offset_x: vx + pad.left,
            image_offset_y: vy + pad.top,
            image_w_active,
            image_h_active,
            is_viewbox: true,
            pad,
        })
    } else if pad_val.is_some() {
        // Case 2: pad without viewbox — auto-compute.
        let canvas_w = iw + pad.left + pad.right;
        let canvas_h = ih + pad.top + pad.bottom;

        Ok(CoordSpace {
            origin_x: 0.0,
            origin_y: 0.0,
            canvas_w,
            canvas_h,
            scale_x: 1.0,
            scale_y: 1.0,
            image_offset_x: pad.left,
            image_offset_y: pad.top,
            image_w_active: iw,
            image_h_active: ih,
            is_viewbox: true,
            pad,
        })
    } else {
        // Case 1: default image space.
        Ok(CoordSpace {
            origin_x: 0.0,
            origin_y: 0.0,
            canvas_w: iw,
            canvas_h: ih,
            scale_x: 1.0,
            scale_y: 1.0,
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            image_w_active: iw,
            image_h_active: ih,
            is_viewbox: false,
            pad: Pad::default(),
        })
    }
}

fn parse_grid(args: &Value, warnings: &mut Vec<String>) -> AgentResult<Option<GridConfig>> {
    let g = match args.get("grid") {
        Some(v) if v.is_object() => v,
        _ => return Ok(None),
    };

    // Accept both integer and float values for divisions — LLMs frequently
    // send `10.0` instead of `10` even when the schema says "integer".
    let x_div_raw = g
        .get("x_divisions")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f.round() as i64)));
    let y_div_raw = g
        .get("y_divisions")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f.round() as i64)));
    let spacing = g.get("spacing").and_then(Value::as_f64);

    let has_divisions = x_div_raw.is_some() || y_div_raw.is_some();
    let has_spacing = spacing.is_some();

    // When both divisions and spacing are provided, prefer divisions (more
    // specific) and warn — LLMs frequently include both despite the schema
    // saying they are mutually exclusive.
    let spacing = if has_divisions && has_spacing {
        warnings.push(
            "grid spacing and divisions are mutually exclusive; \
             ignoring spacing because divisions were also provided"
                .into(),
        );
        None
    } else {
        spacing
    };
    let has_spacing = spacing.is_some();

    if !has_divisions && !has_spacing {
        return Err(AgentError::ValidationError {
            reason: "grid requires either spacing or x_divisions/y_divisions".into(),
        });
    }

    if let Some(s) = spacing
        && s <= 0.0
    {
        return Err(AgentError::ValidationError {
            reason: "grid spacing must be positive".into(),
        });
    }

    if let Some(xd) = x_div_raw
        && xd <= 0
    {
        return Err(AgentError::ValidationError {
            reason: "grid divisions must be positive".into(),
        });
    }
    if let Some(yd) = y_div_raw
        && yd <= 0
    {
        return Err(AgentError::ValidationError {
            reason: "grid divisions must be positive".into(),
        });
    }

    let show_labels = g
        .get("show_labels")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let x_div = x_div_raw.map(|v| v as u32);
    let y_div = y_div_raw.map(|v| v as u32);

    Ok(Some(GridConfig {
        x_divisions: x_div,
        y_divisions: y_div,
        spacing,
        show_labels,
    }))
}

fn parse_crop(args: &Value, cs: &CoordSpace) -> AgentResult<Option<CropConfig>> {
    let c = match args.get("crop") {
        Some(v) if v.is_object() => v,
        _ => return Ok(None),
    };

    let x = c.get("x").and_then(Value::as_f64).unwrap_or(0.0);
    let y = c.get("y").and_then(Value::as_f64).unwrap_or(0.0);
    let width = c.get("width").and_then(Value::as_f64).unwrap_or(0.0);
    let height = c.get("height").and_then(Value::as_f64).unwrap_or(0.0);
    let zoom = c.get("zoom").and_then(Value::as_f64);

    if width <= 0.0 || height <= 0.0 {
        return Err(AgentError::ValidationError {
            reason: "crop width and height must be positive".into(),
        });
    }

    if let Some(z) = zoom
        && (z <= 0.0 || !z.is_finite())
    {
        return Err(AgentError::ValidationError {
            reason: "crop zoom must be a finite positive number".into(),
        });
    }

    // Check intersection with inspectable canvas.
    let canvas_x = cs.origin_x;
    let canvas_y = cs.origin_y;
    let canvas_x2 = cs.origin_x + cs.canvas_w;
    let canvas_y2 = cs.origin_y + cs.canvas_h;

    let crop_x2 = x + width;
    let crop_y2 = y + height;

    let inter_x1 = x.max(canvas_x);
    let inter_y1 = y.max(canvas_y);
    let inter_x2 = crop_x2.min(canvas_x2);
    let inter_y2 = crop_y2.min(canvas_y2);

    if inter_x1 >= inter_x2 || inter_y1 >= inter_y2 {
        return Err(AgentError::ValidationError {
            reason: "crop region does not intersect the inspectable canvas".into(),
        });
    }

    Ok(Some(CropConfig {
        x,
        y,
        width,
        height,
        zoom,
    }))
}

fn parse_probes(args: &Value) -> AgentResult<Vec<ProbeInput>> {
    let arr = match args.get("probes").and_then(Value::as_array) {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let mut probes = Vec::with_capacity(arr.len());
    for (i, p) in arr.iter().enumerate() {
        let id = p
            .get("id")
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| (i + 1).to_string());

        let x = p.get("x").and_then(Value::as_f64).unwrap_or(0.0);
        let y = p.get("y").and_then(Value::as_f64).unwrap_or(0.0);
        let label = p.get("label").and_then(Value::as_str).map(String::from);
        let color = match p.get("color").and_then(Value::as_str) {
            Some(c) => Some(parse_hex_color(c).ok_or_else(|| AgentError::ValidationError {
                reason: format!(
                    "invalid probe color \"{c}\" for probe {id}: expected \"#rrggbb\" hex string"
                ),
            })?),
            None => None,
        };

        probes.push(ProbeInput {
            id,
            x,
            y,
            label,
            color,
        });
    }

    Ok(probes)
}

fn parse_hex_color(s: &str) -> Option<Rgba<u8>> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Rgba([r, g, b, 255]))
}

fn check_canvas_size(w: u32, h: u32) -> AgentResult<()> {
    let total = u64::from(w) * u64::from(h);
    if total > MAX_CANVAS_PIXELS {
        return Err(AgentError::ValidationError {
            reason: format!(
                "requested output image ({w}×{h} = {total} pixels) exceeds the \
                 maximum of {MAX_CANVAS_PIXELS} pixels; reduce zoom, crop size, \
                 or padding"
            ),
        });
    }
    Ok(())
}

fn build_canvas(img: &DynamicImage, cs: &CoordSpace) -> AgentResult<RgbaImage> {
    if !cs.is_viewbox {
        return Ok(img.to_rgba8());
    }

    // Canvas in pixels: we render the full inspectable canvas.
    let canvas_px_w = (cs.canvas_w * cs.scale_x).round() as u32;
    let canvas_px_h = (cs.canvas_h * cs.scale_y).round() as u32;

    check_canvas_size(canvas_px_w, canvas_px_h)?;

    // Neutral gray background for padding.
    let bg = Rgba([240, 240, 240, 255]);
    let mut canvas = RgbaImage::from_pixel(canvas_px_w, canvas_px_h, bg);

    // Composite the source image at the correct offset.
    let offset_px_x = ((cs.image_offset_x - cs.origin_x) * cs.scale_x).round() as i64;
    let offset_px_y = ((cs.image_offset_y - cs.origin_y) * cs.scale_y).round() as i64;

    let rgba_img = img.to_rgba8();
    for (sx, sy, pixel) in rgba_img.enumerate_pixels() {
        let dx = offset_px_x + i64::from(sx);
        let dy = offset_px_y + i64::from(sy);
        if dx >= 0 && dy >= 0 && (dx as u32) < canvas_px_w && (dy as u32) < canvas_px_h {
            canvas.put_pixel(dx as u32, dy as u32, *pixel);
        }
    }

    Ok(canvas)
}

/// Apply crop to the canvas, replacing it with the cropped+zoomed region.
/// Returns the effective zoom factor used.
fn apply_crop(canvas: &mut RgbaImage, crop_cfg: &CropConfig, cs: &CoordSpace) -> AgentResult<f64> {
    let (cw, ch) = (canvas.width(), canvas.height());

    // Map crop region from active coords to pixel coords on the canvas.
    let px_x = ((crop_cfg.x - cs.origin_x) * cs.scale_x).round() as i64;
    let px_y = ((crop_cfg.y - cs.origin_y) * cs.scale_y).round() as i64;
    let px_w = (crop_cfg.width * cs.scale_x).round() as u32;
    let px_h = (crop_cfg.height * cs.scale_y).round() as u32;

    // Determine zoom.
    let zoom = crop_cfg.zoom.unwrap_or_else(|| {
        let longest = px_w.max(px_h) as f64;
        if longest > 0.0 {
            (1024.0 / longest).max(1.0)
        } else {
            1.0
        }
    });

    let out_w = ((px_w as f64) * zoom).round() as u32;
    let out_h = ((px_h as f64) * zoom).round() as u32;

    // Clamp output to reasonable size.
    let out_w = out_w.max(1);
    let out_h = out_h.max(1);

    check_canvas_size(out_w, out_h)?;

    let bg = Rgba([240, 240, 240, 255]);
    let mut cropped = RgbaImage::from_pixel(out_w, out_h, bg);

    // Crop semantics: width/height are exclusive (the crop covers px_w
    // source pixels starting at px_x). Use floor and clamp so we never
    // sample outside the requested region.
    let src_x_max = (px_x + px_w as i64 - 1).min(cw as i64 - 1);
    let src_y_max = (px_y + px_h as i64 - 1).min(ch as i64 - 1);

    for dy in 0..out_h {
        for dx in 0..out_w {
            let src_x = (px_x + (dx as f64 / zoom).floor() as i64).min(src_x_max);
            let src_y = (px_y + (dy as f64 / zoom).floor() as i64).min(src_y_max);

            if src_x >= 0 && src_y >= 0 && (src_x as u32) < cw && (src_y as u32) < ch {
                let pixel = canvas.get_pixel(src_x as u32, src_y as u32);
                cropped.put_pixel(dx, dy, *pixel);
            }
        }
    }

    *canvas = cropped;
    Ok(zoom)
}

fn resolve_theme(theme: &str, canvas: &RgbaImage) -> ThemeColors {
    // Spec: "light" theme = dark gray lines (for bright images).
    //       "dark" theme  = light gray lines (for dark images).
    //       "auto" samples border luminance to choose.
    let use_dark_lines = match theme {
        "light" => true,
        "dark" => false,
        _ => {
            let avg_lum = sample_border_luminance(canvas);
            avg_lum > 128
        }
    };

    if use_dark_lines {
        ThemeColors {
            line: Rgba([40, 40, 40, 128]),
            label_fg: Rgba([40, 40, 40, 255]),
            label_bg: Rgba([255, 255, 255, 180]),
        }
    } else {
        ThemeColors {
            line: Rgba([220, 220, 220, 128]),
            label_fg: Rgba([220, 220, 220, 255]),
            label_bg: Rgba([0, 0, 0, 180]),
        }
    }
}

fn sample_border_luminance(canvas: &RgbaImage) -> u8 {
    let (w, h) = (canvas.width(), canvas.height());
    if w == 0 || h == 0 {
        return 128;
    }

    let mut sum: u64 = 0;
    let mut count: u64 = 0;

    // Sample top and bottom rows, left and right columns.
    let sample = |x: u32, y: u32, sum: &mut u64, count: &mut u64| {
        let p = canvas.get_pixel(x, y);
        // ITU-R BT.601 luminance.
        let lum = (u64::from(p[0]) * 299 + u64::from(p[1]) * 587 + u64::from(p[2]) * 114) / 1000;
        *sum += lum;
        *count += 1;
    };

    for x in 0..w {
        sample(x, 0, &mut sum, &mut count);
        if h > 1 {
            sample(x, h - 1, &mut sum, &mut count);
        }
    }
    for y in 1..h.saturating_sub(1) {
        sample(0, y, &mut sum, &mut count);
        if w > 1 {
            sample(w - 1, y, &mut sum, &mut count);
        }
    }

    if count == 0 { 128 } else { (sum / count) as u8 }
}

fn compute_grid_values(
    canvas_origin: f64,
    canvas_size: f64,
    grid: &GridConfig,
    is_x: bool,
) -> Vec<f64> {
    if let Some(spacing) = grid.spacing {
        // Spacing-based: place lines at multiples of spacing from origin.
        let mut vals = Vec::new();
        let end = canvas_origin + canvas_size;
        // First multiple >= canvas_origin.
        let start_mult = (canvas_origin / spacing).ceil();
        let mut v = start_mult * spacing;
        while v <= end + 1e-9 {
            vals.push(v);
            v += spacing;
        }
        vals
    } else {
        // Division-based.
        let n = if is_x {
            grid.x_divisions.unwrap_or(10)
        } else {
            grid.y_divisions.unwrap_or(10)
        };
        let step = canvas_size / f64::from(n);
        (0..=n)
            .map(|i| canvas_origin + f64::from(i) * step)
            .collect()
    }
}

fn draw_grid(
    canvas: &mut RgbaImage,
    grid: &GridConfig,
    cs: &CoordSpace,
    crop: Option<&CropConfig>,
    effective_zoom: Option<f64>,
    theme: &ThemeColors,
) -> Value {
    let (cw, ch) = (canvas.width(), canvas.height());

    // Determine the active-coordinate range we're drawing over.
    let (view_origin_x, view_origin_y, view_w, view_h) = if let Some(crop_cfg) = crop {
        (crop_cfg.x, crop_cfg.y, crop_cfg.width, crop_cfg.height)
    } else {
        (cs.origin_x, cs.origin_y, cs.canvas_w, cs.canvas_h)
    };

    let x_values = compute_grid_values(view_origin_x, view_w, grid, true);
    let y_values = compute_grid_values(view_origin_y, view_h, grid, false);

    let zoom = effective_zoom.unwrap_or(1.0);

    // Helper: map active coord to output pixel.
    let to_px_x = |ax: f64| -> i32 {
        if crop.is_some() {
            ((ax - view_origin_x) / view_w * cw as f64).round() as i32
        } else {
            ((ax - cs.origin_x) * cs.scale_x * zoom).round() as i32
        }
    };
    let to_px_y = |ay: f64| -> i32 {
        if crop.is_some() {
            ((ay - view_origin_y) / view_h * ch as f64).round() as i32
        } else {
            ((ay - cs.origin_y) * cs.scale_y * zoom).round() as i32
        }
    };

    // Draw vertical grid lines.
    for &xv in &x_values {
        let px = to_px_x(xv);
        if px >= 0 && (px as u32) < cw {
            draw_vline(canvas, px as u32, 0, ch, theme.line);
        }
    }

    // Draw horizontal grid lines.
    for &yv in &y_values {
        let py = to_px_y(yv);
        if py >= 0 && (py as u32) < ch {
            draw_hline(canvas, 0, cw, py as u32, theme.line);
        }
    }

    // Draw labels if enabled.
    if grid.show_labels {
        // X labels along top edge.
        for &xv in &x_values {
            let px = to_px_x(xv);
            if px >= 0 && (px as u32) < cw {
                let label = format!("{}", xv.round() as i64);
                draw_label(canvas, px as u32, 2, &label, theme);
            }
        }

        // Y labels along left edge.
        for &yv in &y_values {
            let py = to_px_y(yv);
            if py >= 0 && (py as u32) < ch {
                let label = format!("{}", yv.round() as i64);
                draw_label(canvas, 2, py as u32, &label, theme);
            }
        }
    }

    // Build metadata — preserve full precision for machine-readable output.
    let mut meta = json!({
        "x_values": x_values,
        "y_values": y_values,
    });

    if grid.spacing.is_some() {
        meta["spacing"] = json!(grid.spacing.unwrap_or(0.0));
    } else {
        meta["x_divisions"] = json!(grid.x_divisions.unwrap_or(10));
        meta["y_divisions"] = json!(grid.y_divisions.unwrap_or(10));
    }

    meta
}

#[allow(clippy::too_many_arguments)]
fn draw_probes(
    canvas: &mut RgbaImage,
    probes: &[ProbeInput],
    src_img: &DynamicImage,
    cs: &CoordSpace,
    crop: Option<&CropConfig>,
    effective_zoom: Option<f64>,
    sample_pixels: bool,
    theme: &ThemeColors,
) -> Value {
    let (cw, ch) = (canvas.width(), canvas.height());
    let (img_w, img_h) = src_img.dimensions();
    let zoom = effective_zoom.unwrap_or(1.0);

    let mut results = Vec::new();

    for probe in probes {
        // Determine if in_canvas and in_image.
        let in_canvas = probe.x >= cs.origin_x
            && probe.x <= cs.origin_x + cs.canvas_w
            && probe.y >= cs.origin_y
            && probe.y <= cs.origin_y + cs.canvas_h;

        // Map to pixel coords.
        let px = ((probe.x - cs.image_offset_x) * cs.scale_x).round() as i64;
        let py = ((probe.y - cs.image_offset_y) * cs.scale_y).round() as i64;
        let in_image = px >= 0 && py >= 0 && (px as u32) < img_w && (py as u32) < img_h;

        // Sample pixel if requested.
        let pixel_color = if sample_pixels && in_image {
            let px_clamped = (px as u32).min(img_w - 1);
            let py_clamped = (py as u32).min(img_h - 1);
            let p = src_img.get_pixel(px_clamped, py_clamped);
            Some(format!("#{:02x}{:02x}{:02x}", p[0], p[1], p[2]))
        } else {
            None
        };

        // Determine if the probe falls within the crop region (when cropping).
        let in_crop = crop.is_none_or(|c| {
            probe.x >= c.x
                && probe.x <= c.x + c.width
                && probe.y >= c.y
                && probe.y <= c.y + c.height
        });

        // Draw crosshair if visible in the output image.
        let visible = in_canvas && in_crop;
        if visible {
            // Map active coords to output canvas pixel.
            let (out_x, out_y) = if let Some(crop_cfg) = crop {
                let ox = ((probe.x - crop_cfg.x) / crop_cfg.width * cw as f64).round() as i32;
                let oy = ((probe.y - crop_cfg.y) / crop_cfg.height * ch as f64).round() as i32;
                (ox, oy)
            } else {
                let ox = ((probe.x - cs.origin_x) * cs.scale_x * zoom).round() as i32;
                let oy = ((probe.y - cs.origin_y) * cs.scale_y * zoom).round() as i32;
                (ox, oy)
            };

            if out_x >= 0 && out_y >= 0 && (out_x as u32) < cw && (out_y as u32) < ch {
                let color = probe.color.unwrap_or(Rgba([255, 50, 50, 255]));
                draw_crosshair(canvas, out_x as u32, out_y as u32, 10, 2, color);

                // Draw label.
                let label_text = probe.label.as_deref().unwrap_or(&probe.id);
                draw_label(
                    canvas,
                    (out_x + 12).max(0) as u32,
                    (out_y - 6).max(0) as u32,
                    label_text,
                    theme,
                );
            }
        }

        let mut entry = json!({
            "id": probe.id,
            "x": probe.x,
            "y": probe.y,
            "in_image": in_image,
            "in_canvas": in_canvas,
        });

        // When a crop is active, include whether the probe is visible in the
        // cropped output so consumers can distinguish "inside the full canvas"
        // from "visible in the returned image".
        if crop.is_some() {
            entry["in_crop"] = json!(in_crop);
        }

        if let Some(ref hex) = pixel_color {
            entry["pixel"] = json!(hex);
        }

        results.push(entry);
    }

    Value::Array(results)
}

// ---------------------------------------------------------------------------
// Drawing primitives (minimal bitmap renderer)
// ---------------------------------------------------------------------------

fn blend_pixel(canvas: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    let (w, h) = (canvas.width(), canvas.height());
    if x >= w || y >= h {
        return;
    }
    let bg = canvas.get_pixel(x, y);
    let alpha = color[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    let r = (color[0] as f32 * alpha + bg[0] as f32 * inv) as u8;
    let g = (color[1] as f32 * alpha + bg[1] as f32 * inv) as u8;
    let b = (color[2] as f32 * alpha + bg[2] as f32 * inv) as u8;
    canvas.put_pixel(x, y, Rgba([r, g, b, 255]));
}

fn draw_vline(canvas: &mut RgbaImage, x: u32, y_start: u32, y_end: u32, color: Rgba<u8>) {
    for y in y_start..y_end {
        blend_pixel(canvas, x, y, color);
    }
}

fn draw_hline(canvas: &mut RgbaImage, x_start: u32, x_end: u32, y: u32, color: Rgba<u8>) {
    for x in x_start..x_end {
        blend_pixel(canvas, x, y, color);
    }
}

fn draw_crosshair(canvas: &mut RgbaImage, cx: u32, cy: u32, arm: u32, gap: u32, color: Rgba<u8>) {
    // Horizontal arms.
    for dx in gap..=arm {
        blend_pixel(canvas, cx.wrapping_add(dx), cy, color);
        if cx >= dx {
            blend_pixel(canvas, cx - dx, cy, color);
        }
    }
    // Vertical arms.
    for dy in gap..=arm {
        blend_pixel(canvas, cx, cy.wrapping_add(dy), color);
        if cy >= dy {
            blend_pixel(canvas, cx, cy - dy, color);
        }
    }
}

// ---------------------------------------------------------------------------
// Minimal bitmap font for labels
// ---------------------------------------------------------------------------

/// 5x7 bitmap font glyphs for printable ASCII.
/// Each glyph is 5 columns x 7 rows, stored as `[u8; 7]` where each u8
/// has the 5 column bits (MSB = leftmost).
const GLYPH_WIDTH: u32 = 5;
const GLYPH_HEIGHT: u32 = 7;

#[rustfmt::skip]
fn glyph_data(ch: char) -> Option<[u8; 7]> {
    Some(match ch {
        // Digits
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        // Uppercase letters
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        // Lowercase letters (rendered as small-caps for legibility at 5x7)
        'a' => [0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111],
        'b' => [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'c' => [0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110],
        'd' => [0b00001, 0b00001, 0b01111, 0b10001, 0b10001, 0b10001, 0b01111],
        'e' => [0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110],
        'f' => [0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000],
        'g' => [0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b01110],
        'h' => [0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
        'i' => [0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110],
        'j' => [0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100],
        'k' => [0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010],
        'l' => [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'm' => [0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001],
        'n' => [0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
        'o' => [0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'p' => [0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        'q' => [0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b00001],
        'r' => [0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000],
        's' => [0b00000, 0b00000, 0b01110, 0b10000, 0b01110, 0b00001, 0b11110],
        't' => [0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110],
        'u' => [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101],
        'v' => [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'w' => [0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010],
        'x' => [0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
        'y' => [0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
        'z' => [0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111],
        // Punctuation and symbols
        ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '#' => [0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '=' => [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000],
        _ => return None,
    })
}

fn draw_label(canvas: &mut RgbaImage, x: u32, y: u32, text: &str, theme: &ThemeColors) {
    let (cw, ch) = (canvas.width(), canvas.height());
    let char_count = text.chars().filter(|c| glyph_data(*c).is_some()).count() as u32;
    if char_count == 0 {
        return;
    }

    let padding: u32 = 2;
    let label_w = char_count * (GLYPH_WIDTH + 1) - 1 + padding * 2;
    let label_h = GLYPH_HEIGHT + padding * 2;

    // Clamp position so label stays on-canvas.
    let lx = x.min(cw.saturating_sub(label_w));
    let ly = y.min(ch.saturating_sub(label_h));

    // Draw background rectangle.
    for dy in 0..label_h {
        for dx in 0..label_w {
            let px = lx + dx;
            let py = ly + dy;
            if px < cw && py < ch {
                blend_pixel(canvas, px, py, theme.label_bg);
            }
        }
    }

    // Draw each character.
    let mut cursor_x = lx + padding;
    for ch_char in text.chars() {
        if let Some(glyph) = glyph_data(ch_char) {
            for (row, &bits) in glyph.iter().enumerate() {
                for col in 0..GLYPH_WIDTH {
                    if bits & (1 << (GLYPH_WIDTH - 1 - col)) != 0 {
                        let px = cursor_x + col;
                        let py = ly + padding + row as u32;
                        if px < cw && py < ch {
                            blend_pixel(canvas, px, py, theme.label_fg);
                        }
                    }
                }
            }
            cursor_x += GLYPH_WIDTH + 1;
        }
    }
}

fn encode_png(canvas: &RgbaImage) -> AgentResult<Vec<u8>> {
    let mut current = canvas.clone();
    let max_attempts = 4;

    for attempt in 0..=max_attempts {
        let mut buf = Vec::new();
        current
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .map_err(|e| AgentError::Io {
                message: format!("failed to encode PNG: {e}"),
            })?;

        if buf.len() <= MAX_IMAGE_BYTES || attempt == max_attempts {
            return Ok(buf);
        }

        // Resize down proportionally to the overshoot.
        let scale = (MAX_IMAGE_BYTES as f64 / buf.len() as f64).sqrt() * 0.9;
        let new_w = ((current.width() as f64) * scale).round() as u32;
        let new_h = ((current.height() as f64) * scale).round() as u32;

        current = image::imageops::resize(
            &current,
            new_w.max(1),
            new_h.max(1),
            image::imageops::FilterType::Lanczos3,
        );
    }

    unreachable!()
}

fn build_coord_space_meta(cs: &CoordSpace) -> Value {
    let mut meta = json!({
        "type": if cs.is_viewbox { "viewbox" } else { "image" },
        "origin": { "x": cs.origin_x, "y": cs.origin_y },
        "size": { "width": cs.canvas_w, "height": cs.canvas_h },
    });

    if cs.is_viewbox {
        meta["pad"] = json!({
            "top": cs.pad.top,
            "right": cs.pad.right,
            "bottom": cs.pad.bottom,
            "left": cs.pad.left,
        });
        meta["image_region"] = json!({
            "x": cs.image_offset_x,
            "y": cs.image_offset_y,
            "width": cs.image_w_active,
            "height": cs.image_h_active,
        });
    }

    meta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff0000"), Some(Rgba([255, 0, 0, 255])));
        assert_eq!(parse_hex_color("00ff00"), Some(Rgba([0, 255, 0, 255])));
        assert_eq!(parse_hex_color("#zz0000"), None);
        assert_eq!(parse_hex_color("#fff"), None);
    }

    #[test]
    fn test_glyph_data_coverage() {
        for ch in
            "0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz ._,:()/#+=".chars()
        {
            assert!(glyph_data(ch).is_some(), "missing glyph for '{ch}'");
        }
        assert!(glyph_data('~').is_none());
    }

    #[test]
    fn test_coordinate_space_default() {
        let args = json!({ "file_path": "test.png" });
        let cs = parse_coordinate_space(&args, 500, 750).expect("valid coordinate space");
        assert!(!cs.is_viewbox);
        assert_eq!(cs.canvas_w, 500.0);
        assert_eq!(cs.canvas_h, 750.0);
        assert_eq!(cs.scale_x, 1.0);
        assert_eq!(cs.scale_y, 1.0);
    }

    #[test]
    fn test_coordinate_space_pad_only() {
        let args = json!({
            "coordinate_space": {
                "pad": { "top": 0.0, "right": 220.0, "bottom": 0.0, "left": 0.0 }
            }
        });
        let cs = parse_coordinate_space(&args, 500, 750).expect("valid coordinate space");
        assert!(cs.is_viewbox);
        assert_eq!(cs.canvas_w, 720.0);
        assert_eq!(cs.canvas_h, 750.0);
        assert_eq!(cs.image_offset_x, 0.0);
        assert_eq!(cs.image_offset_y, 0.0);
        assert_eq!(cs.image_w_active, 500.0);
        assert_eq!(cs.image_h_active, 750.0);
    }

    #[test]
    fn test_coordinate_space_viewbox_with_pad() {
        let args = json!({
            "coordinate_space": {
                "viewbox": { "x": 0.0, "y": 0.0, "width": 720.0, "height": 750.0 },
                "pad": { "top": 0.0, "right": 220.0, "bottom": 0.0, "left": 0.0 }
            }
        });
        let cs = parse_coordinate_space(&args, 500, 750).expect("valid coordinate space");
        assert!(cs.is_viewbox);
        assert_eq!(cs.canvas_w, 720.0);
        assert_eq!(cs.canvas_h, 750.0);
        assert_eq!(cs.image_w_active, 500.0);
        assert_eq!(cs.image_h_active, 750.0);
        // scale_x = 500 / 500 = 1.0
        assert_eq!(cs.scale_x, 1.0);
    }

    #[test]
    fn test_coordinate_space_validation_errors() {
        // Negative pad.
        let args = json!({
            "coordinate_space": {
                "pad": { "top": -1.0, "right": 0.0, "bottom": 0.0, "left": 0.0 }
            }
        });
        assert!(parse_coordinate_space(&args, 100, 100).is_err());

        // Viewbox with zero width.
        let args = json!({
            "coordinate_space": {
                "viewbox": { "x": 0.0, "y": 0.0, "width": 0.0, "height": 100.0 }
            }
        });
        assert!(parse_coordinate_space(&args, 100, 100).is_err());

        // Pad too large for viewbox.
        let args = json!({
            "coordinate_space": {
                "viewbox": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 },
                "pad": { "top": 50.0, "right": 0.0, "bottom": 51.0, "left": 0.0 }
            }
        });
        assert!(parse_coordinate_space(&args, 100, 100).is_err());
    }

    #[test]
    fn test_grid_validation() {
        let mut warnings = Vec::new();

        // Both spacing and divisions — divisions win with a warning.
        let args = json!({ "grid": { "spacing": 50.0, "x_divisions": 10 } });
        let result = parse_grid(&args, &mut warnings).expect("should succeed");
        assert!(result.is_some());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("mutually exclusive"));
        warnings.clear();

        // Neither spacing nor divisions.
        let args = json!({ "grid": {} });
        assert!(parse_grid(&args, &mut warnings).is_err());

        // Zero spacing.
        let args = json!({ "grid": { "spacing": 0.0 } });
        assert!(parse_grid(&args, &mut warnings).is_err());

        // Zero divisions.
        let args = json!({ "grid": { "x_divisions": 0 } });
        assert!(parse_grid(&args, &mut warnings).is_err());

        // Negative divisions.
        let args = json!({ "grid": { "x_divisions": -1, "y_divisions": 5 } });
        assert!(parse_grid(&args, &mut warnings).is_err());

        let args = json!({ "grid": { "y_divisions": -3 } });
        assert!(parse_grid(&args, &mut warnings).is_err());
    }

    #[test]
    fn test_crop_validation() {
        let cs = CoordSpace {
            origin_x: 0.0,
            origin_y: 0.0,
            canvas_w: 500.0,
            canvas_h: 500.0,
            scale_x: 1.0,
            scale_y: 1.0,
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            image_w_active: 500.0,
            image_h_active: 500.0,
            is_viewbox: false,
            pad: Pad::default(),
        };

        // Negative width.
        let args = json!({ "crop": { "x": 0.0, "y": 0.0, "width": -10.0, "height": 100.0 } });
        assert!(parse_crop(&args, &cs).is_err());

        // No intersection.
        let args = json!({ "crop": { "x": 600.0, "y": 600.0, "width": 100.0, "height": 100.0 } });
        assert!(parse_crop(&args, &cs).is_err());

        // Zero zoom.
        let args =
            json!({ "crop": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0, "zoom": 0.0 } });
        assert!(parse_crop(&args, &cs).is_err());

        // Negative zoom.
        let args = json!({ "crop": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0, "zoom": -2.0 } });
        assert!(parse_crop(&args, &cs).is_err());

        // NaN zoom (json doesn't support infinity, but NaN can arrive via direct Value construction).
        let mut args = json!({ "crop": { "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 } });
        args["crop"]["zoom"] = serde_json::Value::from(f64::NAN);
        // NaN is not finite, so if as_f64 returns it, the validation catches it.
        // If as_f64 returns None (json null-like), zoom is None and auto-fit applies.
        // Either way, no crash.
        let _ = parse_crop(&args, &cs);

        // Valid crop.
        let args = json!({ "crop": { "x": 80.0, "y": 100.0, "width": 140.0, "height": 100.0 } });
        assert!(parse_crop(&args, &cs).is_ok());

        // Valid crop with zoom.
        let args = json!({ "crop": { "x": 80.0, "y": 100.0, "width": 140.0, "height": 100.0, "zoom": 2.0 } });
        assert!(parse_crop(&args, &cs).is_ok());
    }

    #[test]
    fn test_compute_grid_values_spacing() {
        let grid = GridConfig {
            x_divisions: None,
            y_divisions: None,
            spacing: Some(50.0),
            show_labels: true,
        };
        let vals = compute_grid_values(0.0, 500.0, &grid, true);
        assert_eq!(vals.len(), 11); // 0, 50, 100, ..., 500
        assert!((vals[0] - 0.0).abs() < 1e-9);
        assert!((vals[10] - 500.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_grid_values_divisions() {
        let grid = GridConfig {
            x_divisions: Some(5),
            y_divisions: Some(5),
            spacing: None,
            show_labels: true,
        };
        let vals = compute_grid_values(0.0, 500.0, &grid, true);
        assert_eq!(vals.len(), 6); // 0, 100, 200, 300, 400, 500
        assert!((vals[0] - 0.0).abs() < 1e-9);
        assert!((vals[5] - 500.0).abs() < 1e-9);
    }

    #[test]
    fn test_blend_pixel() {
        let mut canvas = RgbaImage::from_pixel(2, 2, Rgba([100, 100, 100, 255]));
        blend_pixel(&mut canvas, 0, 0, Rgba([255, 0, 0, 128]));
        let p = canvas.get_pixel(0, 0);
        // ~178, ~50, ~50
        assert!(p[0] > 150);
        assert!(p[1] < 80);
    }

    #[test]
    fn test_definition_is_valid() {
        let def = definition();
        assert_eq!(def.name, "inspect_image");
        assert!(!def.description.is_empty());
    }

    #[test]
    fn test_coord_space_meta_image() {
        let cs = CoordSpace {
            origin_x: 0.0,
            origin_y: 0.0,
            canvas_w: 500.0,
            canvas_h: 750.0,
            scale_x: 1.0,
            scale_y: 1.0,
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            image_w_active: 500.0,
            image_h_active: 750.0,
            is_viewbox: false,
            pad: Pad::default(),
        };
        let meta = build_coord_space_meta(&cs);
        assert_eq!(meta["type"], "image");
        assert!(meta.get("pad").is_none());
    }

    #[test]
    fn test_coord_space_meta_viewbox() {
        let cs = CoordSpace {
            origin_x: 0.0,
            origin_y: 0.0,
            canvas_w: 720.0,
            canvas_h: 750.0,
            scale_x: 1.0,
            scale_y: 1.0,
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            image_w_active: 500.0,
            image_h_active: 750.0,
            is_viewbox: true,
            pad: Pad {
                top: 0.0,
                right: 220.0,
                bottom: 0.0,
                left: 0.0,
            },
        };
        let meta = build_coord_space_meta(&cs);
        assert_eq!(meta["type"], "viewbox");
        assert_eq!(meta["size"]["width"], 720.0);
        assert_eq!(meta["image_region"]["width"], 500.0);
    }
}
