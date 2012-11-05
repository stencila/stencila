test.DefaultIterator <- function(){
  it = iterate( c('a','b','c'))
  checkTrue(it$more())
  checkEquals(it$step(),'a')
  checkEquals(it$step(),'b')
  checkEquals(it$step(),'c')
  checkTrue(!it$more())
}
