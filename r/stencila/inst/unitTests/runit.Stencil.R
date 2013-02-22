test.Stencil.create <- function(){
  Stencil()
}

test.Stencil.render <- function(){

  s <- Stencil()
  s$load('

    <div data-include="id://some_stencil_id"/>

    <div data-include="id://stencil_id">
      <span data-imported="">This should be removed</span>
    </div>

    <div data-include="id://stencil_id" data-select="div.a">
    </div>

    <div data-include="id://stencil_id" data-select="span">
    </div>

    <div data-include="id://stencil_id">
      <span data-replace="#elem">Yo!</span>
      <span data-before="#elem">Before</span>
      <span data-after="#elem">After</span>
      <span data-prepend="#elem">Prepend</span>
      <span data-append="#elem">Append</span>
    </div>

    <div data-include="id://stencil_id" data-param="greeting:greeting_france"/>

  ')
  
  greeting <- 'Hello world!'
  greeting_france <- 'Bonjour!'
  friends <- c('Murray','Dave','Mel')
  fruits <- data.frame(
    name = c('apple','pear','kiwi','perssimon'),
    colour = c('red','green','brown','orange')
  )
  
  c <- Context('.')
  s$render(c)
  cat(s$dump(),file="Stencil.out")
  #checkEquals(s$dump(),'<span data-text="greeting">Hello world!</span>')
}

test.Stencil.render.script <- function(){
  stencil <- Stencil(paste(
    '<script><![CDATA[',
      'a <- 1',
    ']]></script>',
    '<span data-text="a"></span>',sep=''))
  
  checkEquals(paste(
    '<script><![CDATA[',
      'a <- 1',
    ']]></script>',
    '<span data-text="a">1</span>',sep=''),render(stencil))
}

test.Stencil.render.script.error <- function(){
  checkEquals("<script data-error=\"object 'b' not found\">a = b</script>",render("<script>a = b</script>"))
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
        '<li data-text="item">42</li>',
        '<li data-text="item">3.14</li>',
        '<li data-text="item">a</li>',
        '<li data-text="item">string</li>',
      '</ul>',sep=''),
    render(paste(
      '<ul data-for="item:items">',
        '<li data-text="item"></li>',
        '<li>previous</li>',
        '<li>previous</li>',
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
  checkEquals('<div data-if="test"><span data-text="test">previous</span></div>',render(stencil,list(test=FALSE)))
  checkEquals('<div data-if="test" data-active="true"><span data-text="test">TRUE</span></div>',render(stencil,list(test=TRUE)))
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
      '<li data-value="1" />',
      '<li data-value="2" />',
      '<li data-default="" />',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-value="1" data-active="true" />',
      '<li data-value="2" />',
      '<li data-default="" />',
    '</ul>',sep=''),render(stencil,list(a=1)))
            
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-value="1" />',
      '<li data-value="2" data-active="true" />',
      '<li data-default="" />',
    '</ul>',sep=''),render(stencil,list(a=2)))
              
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-value="1" />',
      '<li data-value="2" />',
      '<li data-default="" data-active="true" />',
    '</ul>',sep=''),render(stencil,list(a=99)))
}

test.Stencil.render.switch.no_default <- function(){
  stencil <- Stencil(paste(
    '<ul data-switch="a">',
      '<li data-value="1" />',
      '<li data-value="2" />',
    '</ul>',sep=''))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-value="1" data-active="true" />',
      '<li data-value="2" />',
    '</ul>',sep=''),render(stencil,list(a=1)))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-value="1" />',
      '<li data-value="2" data-active="true" />',
    '</ul>',sep=''),render(stencil,list(a=2)))
  
  checkEquals(paste(
    '<ul data-switch="a">',
      '<li data-value="1" />',
      '<li data-value="2" />',
    '</ul>',sep=''),render(stencil,list(a=99)))
}

test.Stencil.render.switch.no_children <- function(){
  checkEquals('<ul data-switch="a" />',render('<ul data-switch="a" />',list(a='not actually used')))
}


