test.StencilMarkcode.html <- function(){

    checkEquals(
        '<p></p>',
        html(p())
    )
    
    checkEquals(
        '<p>Hello world</p>',
        html(p('Hello world'))
    )

    checkEquals(
        '<p class="special">Hello world</p>',
        html(p(class='special','Hello world'))
    )
    
    checkEquals(
        '<p>2+2=4</p>',
        html(p('2+2=',2+2))
    )
}

test.StencilMarkcode.sink <- function(){

    s <- Stencil()
    s$sink()
    
    p()
    
    s$unsink()

    checkEquals(
        '<p></p>',
        s$content()
    )
    
}
