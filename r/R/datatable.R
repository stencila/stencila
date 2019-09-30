# Functions for interoperability between `Datatable` nodes
# and R `data.frame`s

#' Create a [Datatable()] from a `data.frame`
#'
#' @param df The data frame to convert
#' @aliases as.Datatable.data.frame
#' @export
datatable_from_dataframe <- function(df) {
  Datatable(
    columns = lapply(colnames(df), function(colname) {
      datatable_column_from_object(colname, df[[colname]])
    })
  )
}

#' @export
as.Datatable.data.frame <- datatable_from_dataframe # nolint

#' Create a `data.frame` from a [Datatable()]
#'
#' @param dt The `Datatable` to convert
#' @aliases as.data.frame.Datatable
#' @export
datatable_to_dataframe <- function(dt) {
  data.frame(
    lapply(dt$columns, datatable_column_to_values),
    stringsAsFactors = FALSE
  )
}

#' @export
as.data.frame.Datatable <- function(x, row.names, optional, ...) {
  # Used parameters are necessary to avoid R CMD check warnings
  # regarding S3 method consistency
  datatable_to_dataframe(x)
}

#' Create a [DatatableColumn()] from a R object
#'
#' Because a `factor`'s levels are always a
#' character vector, factors are converted into a
#' column with `schema.items` of type `EnumSchema` with
#' `values` containing the levels.
#'
#' @param name Name of the column
#' @param object The object, usually a `vector`, to generate a schema and values from
datatable_column_from_object <- function(name, object) {
  if (is.factor(object)) {
    sub_schema <- EnumSchema(
      values = levels(object)
    )
    values <- as.character.factor(object)
  } else {
    sub_schema <- switch(
      mode_to_schema_type(mode(object)),
      boolean = BooleanSchema(),
      number = NumberSchema(),
      string = StringSchema()
    )
    values <- object
  }

  DatatableColumn(
    name = name,
    schema = ArraySchema(items = sub_schema),
    values = values
  )
}

#' Create a R `vector` or `factor` from a [DatatableColumn()]
#'
#' @param dtc The [DatatableColumn()] to convert
datatable_column_to_values <- function(dtc) {
  name <- dtc$name
  values <- dtc$values

  schema <- dtc$schema
  if (!is.null(schema)) {
    items <- schema$items
    if (!is.null(items)) {
      type <- items$type
      if (!is.null(type)) {
        if (type == "EnumSchema") {
          values <- factor(values, levels = items$values)
        } else {
          values <- switch(
            type,
            BooleanSchema = as.logical,
            IntegerSchema = as.integer,
            NumberSchema = as.numeric,
            StringSchema = as.character
          )(values)
        }
      }
    }
  }

  result <- list()
  result[[name]] <- values
  result
}
