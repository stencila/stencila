test.Stencil.create <- function(){
  Stencil()
}

test.Stencil.render.code <- function(){
  stencil <- Stencil(paste(
    '<code data-code="r"><![CDATA[',
      'a <- 1',
    ']]></code>',
    '<span data-text="a"></span>',sep=''))
  
  checkEquals(paste(
    '<code data-code="r"><![CDATA[',
      'a <- 1',
    ']]></code>',
    '<span data-text="a">1</span>',sep=''),render(stencil))
}

test.Stencil.render.code.error <- function(){
  checkEquals(
    '<code data-code="r">b<div data-error="true">object \'b\' not found</div></code>',render(
    '<code data-code="r">b</code>'
  ))
}

test.Stencil.render.image <- function(){
    render('<script data-image="svg">plot(1,1)</script>')
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

test.Stencil.render.for <- function(){
  checkEquals(
    paste(
      '<ul data-for="item:items">',
        '<li data-each="true" data-text="item" data-off="true" />',
        '<li data-text="item" data-index="0">42</li>',
        '<li data-text="item" data-index="1">3.14</li>',
        '<li data-text="item" data-index="2">a</li>',
        '<li data-text="item" data-index="3">string</li>',
      '</ul>',sep=''),
    render(paste(
      '<ul data-for="item:items">',
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
  stencil <- Stencil('<div data-if="test"><span data-text="test">previous</span></div>')
  checkEquals('<div data-if="test" data-off="true"><span data-text="test">previous</span></div>',render(stencil,list(test=FALSE)))
  checkEquals('<div data-if="test"><span data-text="test">TRUE</span></div>',render(stencil,list(test=TRUE)))
}

test.Stencil.render.if.else <- function(){
  return('else is not implemented')
}

test.Stencil.render.if.elif <- function(){
  return('elif is not implemented')
  
  stencil <- Stencil(paste(
    '<ul>',
      '<li data-if="a" />',
      '<li data-elif="b" />',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul>',
      '<li data-if="a" />',
      '<li data-elif="b" />',
    '</ul>',sep=''),render(stencil,list(a=FALSE,b=FALSE)))
  checkEquals(paste(
    '<ul>',
      '<li data-if="a" data-active="true" />',
      '<li data-elif="b" />',
    '</ul>',sep=''),render(stencil,list(a=TRUE,b=FALSE)))
  checkEquals(paste(
    '<ul>',
      '<li data-if="a" data-active="true" />',
      '<li data-elif="b" />',
    '</ul>',sep=''),render(stencil,list(a=TRUE,b=TRUE)))
  checkEquals(paste(
    '<ul>',
      '<li data-if="a" />',
      '<li data-elif="b" data-active="true" />',
    '</ul>',sep=''),render(stencil,list(a=FALSE,b=TRUE)))
}

test.Stencil.render.switch <- function(){
  stencil <- Stencil(paste(
    '<ul data-switch="a">',
      '<li data-case="1" />',
      '<li data-case="2" />',
      '<li data-default="" />',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" />',
      '<li data-case="2" data-off="true" />',
      '<li data-default="" data-off="true" />',
    '</ul>',sep=''),render(stencil,list(a=1)))
            
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true" />',
      '<li data-case="2" />',
      '<li data-default="" data-off="true" />',
    '</ul>',sep=''),render(stencil,list(a=2)))
              
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true" />',
      '<li data-case="2" data-off="true" />',
      '<li data-default="" />',
    '</ul>',sep=''),render(stencil,list(a=99)))
}

test.Stencil.render.switch.no_default <- function(){
  stencil <- Stencil(paste(
    '<ul data-switch="a">',
      '<li data-case="1" />',
      '<li data-case="2" />',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" />',
      '<li data-case="2" data-off="true" />',
    '</ul>',sep=''),render(stencil,list(a=1)))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true" />',
      '<li data-case="2" />',
    '</ul>',sep=''),render(stencil,list(a=2)))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-case="1" data-off="true" />',
      '<li data-case="2" data-off="true" />',
    '</ul>',sep=''),render(stencil,list(a=99)))
}

test.Stencil.render.switch.no_children <- function(){
  checkEquals('<ul data-switch="a" />',render('<ul data-switch="a" />',list(a='not actually used')))
}


