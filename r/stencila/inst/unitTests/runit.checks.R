library(stencila)

test.checks <- function(){
  # This test is simply a reference providing examples of the various
  # check functions that are available
  checkEquals(6, factorial(3))
  checkEquals(c(1,2,3), c(1,2,3))
  checkEqualsNumeric(6, factorial(3))
  checkIdentical(6, factorial(3))
  checkTrue(2 + 2 == 4, 'Arithmetic works')
  checkException(log('a'), 'Unable to take the log() of a string')
}


if(0){
    
  q = ds$cursor("SELECT * FROM t1")
  q$fetch()
  
  dt = ds$table("t1")
  
  print(dt)
  str(dt)
  
  dt$rows()
  dt$columns()
  
  dim(dt)
  nrow(dt)
  ncol(dt)
  
  dt$names()
  
  dt$type(0)
  dt$types()
  
  df = as.data.frame(dt)
  df
  is.factor(df$c)
  
  plot(dt)
  
  dt[1,]
  dt[1,1]
  dt[1:1,2:3]
  dt[1,'sales']
  dt[,'sales']
  
  dt[by(year),y=sum(sales),where(region=='E')]
  
  region_current = 'W'
  dt[by(year),y=sum(sales),where(region==region_current)]
  
}





