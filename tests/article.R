source("R/util.R")
source("R/types.R")

article = Article(
  title='',
  authors=list(Person(
    givenNames=list('Jane')
  )),
  content=list(
    Paragraph(content=list('Hello'))
  )
)

aarticle = Article(
  title = 0
)
