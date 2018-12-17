#' @include Encoder.R

#' @export
JsonEncoder <- R6::R6Class("JsonEncoder",
  inherit = Encoder,

  public = list(
    name = function() {
      "json"
    },

    encode = function(instance) {
      members <- list()
      for (name in ls(instance, sorted = FALSE)) {
        if (!is.function(instance[[name]])) members[[name]] <- instance[[name]]
      }
      toString(jsonlite::toJSON(
        members,
        null = "null",
        na = "null",
        dataframe = "columns",
        digits = NA,
        auto_unbox = TRUE,
        force = TRUE
      ))
    },

    decode = function(message, cls) {
      instance <- cls$new()
      members <- jsonlite::fromJSON(message, simplifyDataFrame = FALSE)
      for (name in ls(members)) {
        instance[[name]] <- members[[name]]
      }
      instance
    }
  )
)
