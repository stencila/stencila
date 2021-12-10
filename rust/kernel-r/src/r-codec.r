# Decode JSON to an R value
decode_value <- function(json) {
  fromJSON(json)
}

# Encode an R value to JSON
encode_value <- function(value) {
  converted <- convert_value(value)
  toJSON(converted, null = "null", digits = NA, force = TRUE)
}

# Convert a value prior to encoding
convert_value <- function(value, options = list()) {
  # The order of these if statements is important (since for e.g. a data.frame is a list)
  if (inherits(value, "Entity")) {
    # A Stencila Schema entity so just return it
    value
  } else if (inherits(value, "recordedplot") || inherits(value, "ggplot")) {
    convert_plot(value, options = options)
  } else if (inherits(value, "table")) {
    # The functions `summary` and `table` return class "table" results
    # Currently, just "print" them. In the future, we may convert these to Datatables.
    unbox(paste(utils::capture.output(base::print(value)), collapse = "\n"))
  } else if (is.data.frame(value)) {
    # Decode to a Datatable
    convert_data_frame(value)
  } else if (
    is.null(value) ||
    is.logical(value) ||
    is.numeric(value) ||
    is.character(value) ||
    is.matrix(value) ||
    is.array(value) ||
    is.vector(value) ||
    is.list(value)
  ) {
    # Return value because, for these types, `toJSON()` will convert
    # to the appropriate JSON type e.g. a matrix to an array of arrays
    value
  } else {
    unbox(paste(utils::capture.output(base::print(value)), collapse = "\n"))
  }
}

# Convert a R plot to an `ImageObject`
convert_plot <- function(value, options = list(), format = "png") {
  # Check that a graphics device exists for the requested format
  if (!exists(format)) {
    log$warn(paste("Unsupported format, defaulting to PNG:", format))
    format <- "png"
  }

  # Create a new graphics device for the format, with a temporary path.
  # The tempdir check is needed when forking.
  filename <- tempfile(fileext = paste0(".", format), tmpdir = tempdir(check=TRUE))
  width <- try(as.numeric(options$width))
  height <- try(as.numeric(options$height))

  func <- get(format)
  func(
    filename,
    width = ifelse(is.numeric(width) && length(width) == 1, width, 10),
    height = ifelse(is.numeric(height) && length(width) == 1, height, 10),
    units = "cm",
    res = 150
  )
  base::print(value)
  grDevices::dev.off()

  list(
    type = unbox("ImageObject"),
    contentUrl = unbox(paste0("data:image/", format, ";base64,", base64encode(filename)))
  )
}

# Convert a R `data.frame` to a `Datatable`
convert_data_frame <- function(df) {
  row_names <- attr(df, "row.names")
  if (!identical(row_names, seq_len(nrow(df)))) {
    columns <- list(convert_data_frame_column("name", row_names))
  } else {
    columns <- NULL
  }

  columns <- c(columns, Filter(function(column) !is.null(column), lapply(colnames(df), function(colname) {
    convert_data_frame_column(colname, df[[colname]])
  })))

  list(
    type = unbox("Datatable"),
    columns = columns
  )
}

# Convert a R `vector` to a `DatatableColumn`
#
# Because a `factor`'s levels are always a character vector, factors are converted into a
# column with `validator.items` of type `EnumValidator` with `values` containing the levels.
convert_data_frame_column <- function(name, object) {
  if (is.factor(object)) {
    validator <- list(type = unbox("EnumValidator"), values = levels(object))
    values <- as.character.factor(object)
  } else if (is.logical(object)) {
    validator <- list(type = unbox("BooleanValidator"))
    values <- object
  } else if (is.numeric(object)) {
    validator <- list(type = unbox("NumberValidator"))
    values <- object
  } else if (is.character(object)) {
    validator <- list(type = unbox("StringValidator"))
    values <- object
  } else {
    return(NULL)
  }

  list(
    type = unbox("DatatableColumn"),
    name = unbox(name),
    values = values,
    validator = list(type = unbox("ArrayValidator"), itemsValidator = validator)
  )
}

# Encode a message to JSON
encode_message <- function(message, type) {
  escaped <- gsub('\\"', '\\\\"', message)
  escaped <- gsub('\\n', '\\\\n', escaped)
  paste0('{"type":"CodeError","errorType":"', type, '","errorMessage":"', escaped, '"}')
}
