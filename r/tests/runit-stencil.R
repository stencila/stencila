test.Stencil.create <- function(){
  Stencil()
}

# Check that stencils can call some methods inherited
# from Component
test.Stencil.inherited <- function(){
  s <- Stencil()
  checkEquals(nrow(s$commits()),0)
  s$commit()
  checkEquals(nrow(s$commits()),2)
}

test.Stencil.html <- function(){
  s <- Stencil()

  s$html <- '<p>foo</p>'
  checkEquals('<p>\n\tfoo\n</p>',s$html)
}

test.Stencil.contexts <- function(){
  s <- Stencil()
  checkEquals(length(s$contexts),0)
  s$cila <- 'r\n\ta <- 1\n'
  checkEquals(s$contexts,'r')
}

render <- function(stencil,context=list()){
    if(!('Stencil' %in% class(stencil))){
        stencil <- Stencil(paste("html://",stencil))
    }
    stencil$render(context)
    stencil$html_get(FALSE)
}

test.Stencil.render.exec <- function(){
  render('<pre data-exec="r">a <- 42</pre><span data-text="a"></span>')
}

test.Stencil.render.exec.1 <- function(){
  # At one point in development, when text followed an
  # exec directive, an infinite loop happened (bug in xml.cpp). 
  # This is a regression "test" for that.
  render('<pre data-exec="r">a <- 42</pre>Text after')
}

test.Stencil.render.exec.image <- function(){
  render('<pre data-exec="r format png">plot(1,1)</pre>')
}

test.Stencil.render.text.numeric <- function(){
  checkEquals('<span data-text="number">42</span>',
       render('<span data-text="number">previous</span>',list(number=42)))
  checkEquals('<span data-text="number">3.14</span>',
       render('<span data-text="number">previous</span>',list(number=3.14)))
}

test.Stencil.render.text.string <- function(){
  checkEquals('<span data-text="string">Bonjour</span>',
       render('<span data-text="string">previous</span>',list(string='Bonjour')))
}

test.Stencil.render.text.container <- function(){
  checkEquals('<span data-text="items">42, 3.14, a, string</span>',
       render('<span data-text="items">previous</span>',list(items=list(42, 3.14, 'a', 'string'))))
  checkEquals('<span data-text="items">1, 2, 3, 4, 5</span>',
       render('<span data-text="items">previous</span>',list(items=1:5)))
}

btest.Stencil.render.for <- function(){
  checkEquals(
    paste(
      '<ul data-for="item in items">',
        '<li data-each="true" data-text="item" data-off="true" />',
        '<li data-text="item" data-index="0">42</li>',
        '<li data-text="item" data-index="1">3.14</li>',
        '<li data-text="item" data-index="2">a</li>',
        '<li data-text="item" data-index="3">string</li>',
      '</ul>',sep=''),
    render(paste(
      '<ul data-for="item in items">',
        '<li data-each="true" data-text="item"></li>',
      '</ul>',sep=''),
      list(items=list(42, 3.14, 'a', 'string'))
    )
  )
}

test.Stencil.render.with.list <- function(){
  checkEquals('<div data-with="list"><span data-text="string">Bonjour</span></div>',
       render('<div data-with="list"><span data-text="string">previous</span></div>',list(list=list(string='Bonjour'))))
}

test.Stencil.render.with.data.frame <- function(){
  checkEquals('<div data-with="df"><span data-text="a">1, 2, 3</span></div>',
       render('<div data-with="df"><span data-text="a">previous</span></div>',list(df=data.frame(a=1:3))))
}

test.Stencil.render.if <- function(){
  stencil <- Stencil('html://<div data-if="test"><span data-text="test">previous</span></div>')
  checkEquals('<div data-if="test" data-off="true"><span data-text="test">previous</span></div>',render(stencil,list(test=FALSE)))
  checkEquals('<div data-if="test"><span data-text="test">TRUE</span></div>',render(stencil,list(test=TRUE)))
}

test.Stencil.render.if.else <- function(){
  return('else is not implemented')
}

test.Stencil.render.if.elif <- function(){
  stencil <- Stencil(paste(
    'html://<ul>',
      '<li data-if="a" />',
      '<li data-elif="b" />',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul>',
      '<li data-if="a" data-off="true"></li>',
      '<li data-elif="b" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=FALSE,b=FALSE)))

  checkEquals(paste(
    '<ul>',
      '<li data-if="a"></li>',
      '<li data-elif="b" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=TRUE,b=FALSE)))

  checkEquals(paste(
    '<ul>',
      '<li data-if="a"></li>',
      '<li data-elif="b" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=TRUE,b=TRUE)))

  checkEquals(paste(
    '<ul>',
      '<li data-if="a" data-off="true"></li>',
      '<li data-elif="b"></li>',
    '</ul>',sep=''),render(stencil,list(a=FALSE,b=TRUE)))
}

test.Stencil.render.switch <- function(){
  stencil <- Stencil(paste(
    'html://<ul data-switch="a">',
      '<li data-case="1"></li>',
      '<li data-case="2"></li>',
      '<li data-default=""></li>',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1"></li>',
      '<li data-case="2" data-off="true"></li>',
      '<li data-default="" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=1)))
            
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true"></li>',
      '<li data-case="2"></li>',
      '<li data-default="" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=2)))
              
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true"></li>',
      '<li data-case="2" data-off="true"></li>',
      '<li data-default=""></li>',
    '</ul>',sep=''),render(stencil,list(a=99)))
}

test.Stencil.render.switch.no_default <- function(){
  stencil <- Stencil(paste(
    'html://<ul data-switch="a">',
      '<li data-case="1"></li>',
      '<li data-case="2"></li>',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1"></li>',
      '<li data-case="2" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=1)))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true"></li>',
      '<li data-case="2"></li>',
    '</ul>',sep=''),render(stencil,list(a=2)))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true"></li>',
      '<li data-case="2" data-off="true"></li>',
    '</ul>',sep=''),render(stencil,list(a=99)))
}

test.Stencil.render.switch.no_children <- function(){
  checkEquals('<ul data-switch="a"></ul>',render('<ul data-switch="a"></ul>',list(a='not actually used')))
}


