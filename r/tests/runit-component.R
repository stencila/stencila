test.Component.create <- function(){
  Component()
}

test.Component.commit <- function(){
  c = Component()
  c$commit("Updated the component")
  checkEquals(c$commits()[1,3],"Updated the component")
}
