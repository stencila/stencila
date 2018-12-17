#' @include JsonEncoder.R

#' @export
JsonGzipEncoder <- R6::R6Class("JsonGzipEncoder",
  inherit = JsonEncoder,

  public = list(
    name = function() {
      "json+gzip"
    },

    encode = function(instance) {
      memCompress(super$encode(instance))
    },

    decode = function(message, cls) {
      super$decode(memDecompress(message, "gzip", asChar = TRUE), cls)
    }
  )
)
