test.Workspace.set_get <- function(){
  #Check that set and get are properly scoped
  co <- Workspace()
  #Enter an anonymous block, set and get a variable
  co$enter()
  co$set('x','2')
  checkEquals(co$get('x'),2)
  #Enter another anonymous blockset a variable and calculate something
  co$enter()
  co$set('y','3')
  checkEquals(co$get('x*y'),6)
  #Exit 2nd block, y should no longer be available
  co$exit()
  checkException(co$get('y'),"object 'y' not found")
  #Exit first block, x should no longer be available
  co$exit()
  checkException(co$get('x'),"object 'x' not found")
}

test.Workspace.text <- function(){
  #Check text method
  a <- 1
  b <- "b"
  c <- c(1,2,3)
  d <- c("a","b","c")
  co <- Workspace('.')
  checkEquals(co$text('a'),'1')
  checkEquals(co$text('b'),'b')
  checkEquals(co$text('c'),'1, 2, 3')
  checkEquals(co$text('d'),'a, b, c')
}

test.Workspace.enter_exit <- function(){
  #Check nested entry into environments, data.frames and lists
  
  #Set up some object that will be entered
  a <- 'A1'
  c2 <- data.frame(a='A2',stringsAsFactors=F)
  c3 <- list(
    a = 'A3',
    b = data.frame(
      a = 'A4',
      stringsAsFactors=F
    ),
    d = 'D3'
  )
  
  co <- Workspace('.')
  
  checkEquals(co$get('a'),"A1")
  
  co$enter('c2')
  checkEquals(co$get('a'),"A2")
  
  co$enter('c3')
  checkEquals(co$get('a'),"A3")
  checkEquals(co$get('d'),"D3")
  
  co$enter('b')
  checkEquals(co$get('a'),"A4")
  
  co$exit()
  checkEquals(co$get('a'),"A3")
  
  co$exit()
  checkEquals(co$get('a'),"A2")
  
  co$exit()
  checkEquals(co$get('a'),"A1")
  
  checkException(co$get('d'),"object 'd' not found")
}

test.Workspace.begin_step <- function(){
  # Test looping over items in a container
  items <- c('a','b','c')
  
  co <- Workspace('.')
  
  co$begin('item','items')
  checkEquals(co$get('item'),'a')
  
  co$step() 
  checkEquals(co$get('item'),'b')
  
  co$step() 
  checkEquals(co$get('item'),'c')
  
  co$step()
  checkException(co$get('item'),"object 'item' not found")
}
