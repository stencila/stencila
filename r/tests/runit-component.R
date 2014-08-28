test.Component.create <- function(){
  Component()
}

test.Component.commit <- function(){
  c = Component()
  c$commit("Updated the component")
  checkEquals(c$commits()[1,2],"Updated the component")
}
