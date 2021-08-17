# Generate a list of functions in the packages on the
# default R `search()` list.
#
# Run using `RScript use-ignore.R`
cat(
  paste(
    '"', 
    sort(
      Reduce(
        function(prev, pkg) c(prev, ls(paste0("package:", pkg))),
        c("base", "methods", "utils", "grDevices", "graphics", "stats"),
        character()
      )
    ),
    '"',
    sep="",
    collapse=",\n"
  )
)
