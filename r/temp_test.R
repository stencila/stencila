install.packages('builds/stencila_latest.tar.gz',repo=NULL)
library(stencila)

a = 'A1' 
  
s = Stencil()
s$load('<div data-text="a"></div>')
s$render()
s$dump()
