use winnow::{
    ModalResult, Parser,
    ascii::{digit1, multispace0},
    combinator::{alt, eof},
    token::literal,
};

use thiserror::Error;

const ROW_KEYWORD: &str = "row";
const INVALID_COMBINED_FORM: &str = "invalid combined form";
const INVALID_MAP_CELL: &str = "invalid map cell";
const EMPTY_TOKEN: &str = "empty token";
const INVALID_GAP_TOKEN: &str = "invalid gap token";
const INVALID_COLUMN_TOKEN: &str = "invalid column token";

/// Parse a layout specification string into a [`Layout`].
///
/// Supported forms currently include:
///
/// - integer column counts such as `"2"`
/// - the row preset `"row"`
/// - explicit column specifications such as `"30,70"` or `"40,g20,40"`
/// - layout maps such as `"a,b|a,c"`
/// - combined explicit-column and map forms such as `"30,70:a,b|a,c"`
///
/// Leading and trailing whitespace is ignored, but the parser requires full
/// consumption of the trimmed input.
///
/// # Caller responsibilities
///
/// When using [`Layout::Row`], callers should resolve it with
/// [`Layout::resolve_row`] using the actual number of subfigures before
/// rendering.
///
/// # Examples
///
/// ```
/// use stencila_layout_lang::{Columns, Layout, Placement, parse};
///
/// assert_eq!(parse("row"), Ok(Layout::Row));
/// assert_eq!(
///     parse("30,70:a,b|a,c"),
///     Ok(Layout::Map {
///         columns: Columns {
///             widths: vec![30, 70],
///             gaps: vec![None],
///         },
///         placements: vec![
///             Placement {
///                 label: 'a',
///                 col: 0,
///                 row: 0,
///                 col_span: 1,
///                 row_span: 2,
///             },
///             Placement {
///                 label: 'b',
///                 col: 1,
///                 row: 0,
///                 col_span: 1,
///                 row_span: 1,
///             },
///             Placement {
///                 label: 'c',
///                 col: 1,
///                 row: 1,
///                 col_span: 1,
///                 row_span: 1,
///             },
///         ],
///     })
/// );
/// ```
pub fn parse(input: &str) -> Result<Layout, LayoutError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(LayoutError::EmptyInput);
    }

    if trimmed.contains(':') {
        return parse_form4_str(trimmed);
    }

    match detect_parse_form(trimmed) {
        ParseForm::Standard => parse_standard_layout(trimmed),
        ParseForm::ColumnSpec => parse_form2_str(trimmed),
        ParseForm::Map => parse_form3_str(trimmed),
    }
}

/// The column structure of a layout, specifying relative widths and optional gaps.
///
/// Each entry in [`Columns::gaps`] corresponds to the gap *between* adjacent
/// columns. A `None` value means the gap uses the default spacing; `Some(n)`
/// specifies an explicit relative gap width.
///
/// # Examples
///
/// ```
/// use stencila_layout_lang::Columns;
///
/// let columns = Columns::equal(3);
///
/// assert_eq!(columns.widths, vec![1, 1, 1]);
/// assert_eq!(columns.gaps, vec![None, None]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Columns {
    /// Relative width for each column from left to right.
    pub widths: Vec<u32>,

    /// Optional relative gap widths between adjacent columns.
    ///
    /// This vector has one fewer entries than [`Columns::widths`] for a valid
    /// explicit grid. Each item describes the gap after the column at the same
    /// index.
    pub gaps: Vec<Option<u32>>,
}

impl Columns {
    /// Create a column layout with `n` equal-width columns and default gaps.
    ///
    /// # Examples
    ///
    /// ```
    /// use stencila_layout_lang::Columns;
    ///
    /// let columns = Columns::equal(2);
    ///
    /// assert_eq!(columns.widths, vec![1, 1]);
    /// assert_eq!(columns.gaps, vec![None]);
    /// ```
    pub fn equal(n: usize) -> Self {
        Self {
            widths: vec![1; n],
            gaps: vec![None; n.saturating_sub(1)],
        }
    }

    /// Return the number of columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use stencila_layout_lang::Columns;
    ///
    /// assert_eq!(Columns::equal(4).column_count(), 4);
    /// ```
    pub fn column_count(&self) -> usize {
        self.widths.len()
    }
}

/// A single subfigure placement within a grid layout.
///
/// A placement identifies the top-left grid cell occupied by a labeled item
/// together with its horizontal and vertical span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    /// The subfigure label represented by this placement.
    pub label: char,

    /// Zero-based starting column.
    pub col: u32,

    /// Zero-based starting row.
    pub row: u32,

    /// Number of columns spanned by the placement.
    pub col_span: u32,

    /// Number of rows spanned by the placement.
    pub row_span: u32,
}

/// The layout specification for a figure's subfigures.
///
/// - `Auto` — the renderer decides placement automatically, optionally using an
///   explicit column specification.
/// - `Row` — subfigures are placed in a single row. Call [`Layout::resolve_row`]
///   to convert this into a concrete `Map` layout before rendering. The caller is
///   responsible for ensuring the subfigure count matches the intended labels.
/// - `Map` — an explicit grid layout with column definitions and placements.
///
/// # Examples
///
/// ```
/// use stencila_layout_lang::{Columns, Layout};
///
/// let layout = Layout::Auto {
///     columns: Columns::equal(2),
/// };
///
/// assert_eq!(layout.column_count(), Some(2));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    /// Automatic placement with an explicit column structure.
    Auto {
        /// The column structure to use when automatically placing items.
        columns: Columns,
    },

    /// A preset single-row layout.
    Row,

    /// A fully explicit grid layout.
    Map {
        /// The column structure for the grid.
        columns: Columns,

        /// Placements for each labeled subfigure in first-appearance order.
        placements: Vec<Placement>,
    },
}

impl Layout {
    /// Return the number of columns if this layout defines an explicit grid.
    ///
    /// Returns `None` only for [`Layout::Row`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stencila_layout_lang::{Columns, Layout};
    ///
    /// assert_eq!(Layout::Row.column_count(), None);
    /// assert_eq!(Layout::Auto { columns: Columns::equal(3) }.column_count(), Some(3));
    /// ```
    pub fn column_count(&self) -> Option<usize> {
        match self {
            Self::Auto { columns } => Some(columns.column_count()),
            Self::Map { columns, .. } => Some(columns.column_count()),
            Self::Row => None,
        }
    }

    /// Resolve a `Row` layout into a concrete `Map` with equal columns,
    /// or pass through `Auto` and `Map` unchanged.
    ///
    /// The `n` parameter is the number of subfigures. For `Row`, this
    /// produces a single-row grid with `n` equal columns and one placement
    /// per column.
    ///
    /// Callers are responsible for ensuring `n` matches the number of
    /// subfigures being rendered.
    ///
    /// # Examples
    ///
    /// ```
    /// use stencila_layout_lang::{Columns, Layout};
    ///
    /// let resolved = Layout::Row.resolve_row(2);
    ///
    /// assert_eq!(resolved.column_count(), Some(2));
    /// assert_eq!(resolved, Layout::Map {
    ///     columns: Columns::equal(2),
    ///     placements: vec![
    ///         stencila_layout_lang::Placement {
    ///             label: 'a',
    ///             col: 0,
    ///             row: 0,
    ///             col_span: 1,
    ///             row_span: 1,
    ///         },
    ///         stencila_layout_lang::Placement {
    ///             label: 'b',
    ///             col: 1,
    ///             row: 0,
    ///             col_span: 1,
    ///             row_span: 1,
    ///         },
    ///     ],
    /// });
    /// ```
    pub fn resolve_row(self, n: usize) -> Self {
        match self {
            Self::Row => {
                let columns = Columns::equal(n);
                let placements = (0..n).map(row_placement).collect();
                Self::Map {
                    columns,
                    placements,
                }
            }
            other => other,
        }
    }
}

/// Errors that can occur when parsing or validating a layout specification.
///
/// Parsing errors are reported as [`LayoutError::Parse`]. Semantic validation
/// errors, such as non-rectangular spans or invalid gap placement, are reported
/// with more specific variants.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LayoutError {
    /// The input could not be tokenized or fully parsed.
    #[error("parse error: {0}")]
    Parse(String),

    /// The input contained no layout specification after trimming whitespace.
    #[error("layout input is empty")]
    EmptyInput,

    /// An explicit column width of zero was encountered.
    #[error("column width must not be zero at position {position}")]
    ZeroColumn { position: usize },

    /// An explicit gap width of zero was encountered.
    #[error("gap width must not be zero at position {position}")]
    ZeroGap { position: usize },

    /// A gap token appeared before the first column, after the last column, or adjacent to another gap.
    #[error("gap specified at invalid position {position}")]
    InvalidGapPosition { position: usize },

    /// A map row had a different number of cells from the first row.
    #[error("inconsistent row width: expected {expected}, got {actual}")]
    InconsistentRowWidth { expected: usize, actual: usize },

    /// Repeated cells for a label did not form a solid rectangle.
    #[error("placement span for label '{label}' is not rectangular")]
    NonRectangularSpan { label: char },

    /// A combined column specification and map used different column counts.
    #[error("column count mismatch: expected {expected}, got {actual}")]
    ColumnCountMismatch { expected: usize, actual: usize },

    /// A combined form used a prefix that is not allowed with a map.
    #[error("invalid combination of layout forms: {form}")]
    InvalidCombinedForm { form: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawLayout {
    columns: RawColumnSpec,
}

impl RawLayout {
    fn into_layout(self) -> Result<Layout, LayoutError> {
        Ok(Layout::Auto {
            columns: self.columns.into_columns()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseForm {
    Standard,
    ColumnSpec,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombinedFormError {
    RowPreset,
    IntegerColumnCount,
}

impl CombinedFormError {
    fn into_layout_error(self) -> LayoutError {
        let form = match self {
            Self::RowPreset => "row preset",
            Self::IntegerColumnCount => "integer column count",
        };

        LayoutError::InvalidCombinedForm { form: form.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RawColumnSpec {
    Count(usize),
    Tokens(Vec<ColumnToken>),
}

impl RawColumnSpec {
    fn into_columns(self) -> Result<Columns, LayoutError> {
        match self {
            Self::Count(count) => Ok(Columns::equal(count)),
            Self::Tokens(tokens) => tokens_into_columns(tokens),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ColumnToken {
    Count(usize),
    Gap(usize),
}

fn detect_parse_form(input: &str) -> ParseForm {
    if input == ROW_KEYWORD {
        ParseForm::Standard
    } else if input
        .chars()
        .all(|char| char.is_ascii_digit() || matches!(char, 'g' | ',' | ' ' | '\t'))
        && (input.contains(',') || input.starts_with('g'))
    {
        ParseForm::ColumnSpec
    } else if input.contains('|')
        || input
            .chars()
            .any(|char| char.is_ascii_lowercase() || char == '.')
    {
        ParseForm::Map
    } else {
        ParseForm::Standard
    }
}

fn parse_standard_layout(input: &str) -> Result<Layout, LayoutError> {
    let mut remaining = input;
    layout
        .parse_next(&mut remaining)
        .map_err(|err| LayoutError::Parse(first_error_line(&err.to_string())))
}

fn layout(input: &mut &str) -> ModalResult<Layout> {
    (multispace0, alt((parse_row, parse_form1)), multispace0, eof)
        .map(|(_, layout, _, _)| layout)
        .parse_next(input)
}

fn parse_row(input: &mut &str) -> ModalResult<Layout> {
    literal(ROW_KEYWORD).map(|_| Layout::Row).parse_next(input)
}

fn parse_form1(input: &mut &str) -> ModalResult<Layout> {
    parse_column_count
        .map(|count| RawLayout {
            columns: RawColumnSpec::Count(count),
        })
        .try_map(RawLayout::into_layout)
        .parse_next(input)
}

fn parse_column_count(input: &mut &str) -> ModalResult<usize> {
    digit1.try_map(str::parse::<usize>).parse_next(input)
}

fn parse_form2_str(input: &str) -> Result<Layout, LayoutError> {
    let mut tokens = Vec::new();

    for part in input.split(',') {
        let token = part.trim();

        if token.is_empty() {
            return Err(parse_error(EMPTY_TOKEN));
        }

        if let Some(gap) = token.strip_prefix('g') {
            if gap.is_empty() || !gap.chars().all(|char| char.is_ascii_digit()) {
                return Err(parse_error(INVALID_GAP_TOKEN));
            }

            tokens.push(ColumnToken::Gap(
                gap.parse().map_err(|_| parse_error(INVALID_GAP_TOKEN))?,
            ));
        } else if token.chars().all(|char| char.is_ascii_digit()) {
            tokens.push(ColumnToken::Count(
                token
                    .parse()
                    .map_err(|_| parse_error(INVALID_COLUMN_TOKEN))?,
            ));
        } else {
            return Err(parse_error(INVALID_COLUMN_TOKEN));
        }
    }

    RawLayout {
        columns: RawColumnSpec::Tokens(tokens),
    }
    .into_layout()
}

fn parse_form3_str(input: &str) -> Result<Layout, LayoutError> {
    let rows: Vec<Vec<char>> = input
        .split('|')
        .map(parse_map_row)
        .collect::<Result<_, _>>()?;

    let expected = validate_row_widths(&rows)?;

    let placements = collect_map_placements(&rows)?;

    Ok(Layout::Map {
        columns: Columns::equal(expected),
        placements,
    })
}

fn detect_invalid_combined_form(column_spec: &str) -> Option<CombinedFormError> {
    if column_spec == ROW_KEYWORD {
        Some(CombinedFormError::RowPreset)
    } else if column_spec.chars().all(|char| char.is_ascii_digit()) {
        Some(CombinedFormError::IntegerColumnCount)
    } else {
        None
    }
}

fn validate_row_widths(rows: &[Vec<char>]) -> Result<usize, LayoutError> {
    let expected = rows.first().map(Vec::len).unwrap_or(0);

    if let Some(actual) = rows.iter().map(Vec::len).find(|actual| *actual != expected) {
        Err(LayoutError::InconsistentRowWidth { expected, actual })
    } else {
        Ok(expected)
    }
}

fn parse_form4_str(input: &str) -> Result<Layout, LayoutError> {
    let (column_spec, map) = input
        .split_once(':')
        .ok_or_else(|| LayoutError::Parse(INVALID_COMBINED_FORM.into()))?;

    let column_spec = column_spec.trim();
    let map = map.trim();

    if let Some(error) = detect_invalid_combined_form(column_spec) {
        return Err(error.into_layout_error());
    }

    let columns = match parse_form2_str(column_spec)? {
        Layout::Auto { columns } => columns,
        _ => return Err(LayoutError::Parse(INVALID_COMBINED_FORM.into())),
    };

    let layout = parse_form3_str(map)?;
    let Layout::Map {
        columns: map_columns,
        placements,
    } = layout
    else {
        return Err(LayoutError::Parse(INVALID_COMBINED_FORM.into()));
    };

    let expected = columns.column_count();
    let actual = map_columns.column_count();
    if expected != actual {
        return Err(LayoutError::ColumnCountMismatch { expected, actual });
    }

    Ok(Layout::Map {
        columns,
        placements,
    })
}

fn parse_map_row(row: &str) -> Result<Vec<char>, LayoutError> {
    row.split(',').map(parse_map_cell).collect()
}

fn parse_map_cell(cell: &str) -> Result<char, LayoutError> {
    let cell = cell.trim();

    match cell {
        "." => Ok('.'),
        _ if cell.len() == 1 => cell
            .chars()
            .next()
            .filter(|char| char.is_ascii_lowercase())
            .ok_or_else(|| LayoutError::Parse(INVALID_MAP_CELL.into())),
        _ => Err(LayoutError::Parse(INVALID_MAP_CELL.into())),
    }
}

fn row_placement(index: usize) -> Placement {
    Placement {
        label: char::from(b'a' + index as u8),
        col: index as u32,
        row: 0,
        col_span: 1,
        row_span: 1,
    }
}

fn collect_map_placements(rows: &[Vec<char>]) -> Result<Vec<Placement>, LayoutError> {
    let mut labels = Vec::new();

    for row in rows {
        for &label in row {
            if label != '.' && !labels.contains(&label) {
                labels.push(label);
            }
        }
    }

    labels
        .into_iter()
        .map(|label| derive_placement(rows, label))
        .collect()
}

fn derive_placement(rows: &[Vec<char>], label: char) -> Result<Placement, LayoutError> {
    let cells: Vec<(usize, usize)> = rows
        .iter()
        .enumerate()
        .flat_map(|(row, values)| {
            values
                .iter()
                .enumerate()
                .filter_map(move |(col, value)| (*value == label).then_some((row, col)))
        })
        .collect();

    let min_row = cells.iter().map(|(row, _)| *row).min().unwrap_or(0);
    let max_row = cells.iter().map(|(row, _)| *row).max().unwrap_or(0);
    let min_col = cells.iter().map(|(_, col)| *col).min().unwrap_or(0);
    let max_col = cells.iter().map(|(_, col)| *col).max().unwrap_or(0);

    ensure_rectangular_span(rows, label, min_row, max_row, min_col, max_col)?;

    Ok(Placement {
        label,
        col: min_col as u32,
        row: min_row as u32,
        col_span: (max_col - min_col + 1) as u32,
        row_span: (max_row - min_row + 1) as u32,
    })
}

fn ensure_rectangular_span(
    rows: &[Vec<char>],
    label: char,
    min_row: usize,
    max_row: usize,
    min_col: usize,
    max_col: usize,
) -> Result<(), LayoutError> {
    for values in rows.iter().take(max_row + 1).skip(min_row) {
        for value in values.iter().take(max_col + 1).skip(min_col) {
            if *value != label {
                return Err(LayoutError::NonRectangularSpan { label });
            }
        }
    }

    Ok(())
}

fn tokens_into_columns(tokens: Vec<ColumnToken>) -> Result<Columns, LayoutError> {
    let mut widths = Vec::new();
    let mut gaps = Vec::new();
    let mut expect_column = true;

    for (index, token) in tokens.into_iter().enumerate() {
        let position = index + 1;
        match token {
            ColumnToken::Count(width) => {
                let width = validate_column_width(width, position)?;

                if !expect_column {
                    gaps.push(None);
                }

                widths.push(width);
                expect_column = false;
            }
            ColumnToken::Gap(_) if expect_column => {
                return Err(LayoutError::InvalidGapPosition { position });
            }
            ColumnToken::Gap(gap) => {
                gaps.push(Some(validate_gap_width(gap, position)?));
                expect_column = true;
            }
        }
    }

    if expect_column {
        return Err(LayoutError::InvalidGapPosition {
            position: widths.len() + gaps.len(),
        });
    }

    Ok(Columns { widths, gaps })
}

fn validate_column_width(width: usize, position: usize) -> Result<u32, LayoutError> {
    if width == 0 {
        Err(LayoutError::ZeroColumn { position })
    } else {
        Ok(width as u32)
    }
}

fn validate_gap_width(gap: usize, position: usize) -> Result<u32, LayoutError> {
    if gap == 0 {
        Err(LayoutError::ZeroGap { position })
    } else {
        Ok(gap as u32)
    }
}

fn first_error_line(message: &str) -> String {
    message.lines().next().unwrap_or("parse error").to_string()
}

fn parse_error(message: &'static str) -> LayoutError {
    LayoutError::Parse(message.into())
}
