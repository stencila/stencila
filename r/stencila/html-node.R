#' @include extension.R
NULL

#' A HTML node
#'
#' Used primarily for retrieving elements of a stencil
#' using stencil methods `select` and `filter`. No explicit
#' constructor is provided.
#'
#' @name HtmlNode
#' @export
setRefClass(
	'HtmlNode',
    contains = 'Extension',
	methods = list(
		attr = function(name) method_(.self,'HtmlNode_attr',name),
		text = function() method_(.self,'HtmlNode_text'),
        select = function(selector) method_(.self,'HtmlNode_select',selector)
	)
)
