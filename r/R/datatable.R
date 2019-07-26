
#' Create a \code{\link{Datatable}} from a \code{data.frame}
#'
#' @name from_dataframe
#' @aliases as.Datatable.data.frame
#' @export
from_dataframe <- function(df){
  Datatable(
    columns = lapply(colnames(df), function(colname) {
      values <- df[[colname]]
      DatatableColumn(
        name = colname,
        schema = DatatableColumnSchema(
          items = list(
            type = node_type(values)
          )
        ),
        values = values
      )
    })
  )
}

as.Datatable.data.frame <- from_dataframe

#' Create a \code{data.frame} from a \code{\link{Datatable}}
#'
#' @name to_dataframe
#' @aliases as.data.frame.Datatable
#' @export
to_dataframe <- function(df){
  # TODO: Implement it!
}

as.data.frame.Datatable <- to_dataframe
