
#' Create a [Datatable()] from a `data.frame`
#'
#' @param df The data frame to convert
#' @aliases as.Datatable.data.frame
#' @export
datatable_from_dataframe <- function(df){
  Datatable(
    columns = lapply(colnames(df), function(colname) {
      datatable_column_from_object(colname, df[[colname]])
    })
  )
}

#' @export
as.Datatable.data.frame <- datatable_from_dataframe

#' Create a `data.frame` from a [Datatable()]
#'
#' @param dt The `Datatable` to convert
#' @aliases as.data.frame.Datatable
#' @export
datatable_to_dataframe <- function(dt){
  data.frame(
    lapply(dt$columns, datatable_column_to_values),
    stringsAsFactors = FALSE
  )
}

#' @export
as.data.frame.Datatable <- datatable_to_dataframe

#' Create a [DatatableColumn()] from a R object
#'
#' Because a `factor`'s levels are always a
#' character vector, factors are converted into a
#' columns with `items` of type `string` with
#' a `enum` containing the levels.
#'
#' @param name Name of the column
#' @param object The object, usually a `vector`, to generate a schema and values from
datatable_column_from_object <- function(name, object) {
  if (is.factor(object)) {
    items <- list(
      type = as_scalar("string"),
      enum = levels(object)
    )
    values <- as.character.factor(object)
  } else {
    items <- list(
      type = as_scalar(mode_to_schema_type(mode(object)))
    )
    values <- object
  }

  DatatableColumn(
    name = name,
    schema = DatatableColumnSchema(
      items = items
    ),
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
        mode <- schema_type_to_mode(type)
      }
      enum <- items$enum
      if (!is.null(enum)) {
        values <- factor(values, levels = enum)
      }
    }
  }

  result <- list()
  result[[name]] <- values
  result
}
