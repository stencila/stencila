test.Component.create <- function(){
  Component()
}

test.Component.title <- function(){
  c = Component()
  c$title("foo")
  checkEquals(c$title(),"foo")
}

test.Component.description <- function(){
  c = Component()
  c$description("foo")
  checkEquals(c$description(),"foo")
}

test.Component.keywords <- function(){
  c = Component()
  c$keywords(c("foo","bar"))
  checkEquals(c$keywords(),c("foo","bar"))
}

test.Component.commit <- function(){
  c = Component()
  c$commit("Updated the component")
  checkEquals(c$log()[1,2],"Updated the component")
}
