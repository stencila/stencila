write.csv(data, "path1")

write.table(data, file = "path2")

foo <- function () {
    write.delim(data, file = "path3")
}

# These should be ignored

write.table(data, ignore)

write.csv(data, file = ignore)
