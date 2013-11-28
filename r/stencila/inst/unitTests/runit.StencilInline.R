test.StencilInline.interface <- function(){
    s <- Stencil()
    s$target()
    
    p('Hello world!')
    
    s$untarget()

    checkEquals(
        '<p>Hello world!</p>',
        s$html()
    )
}

test.StencilInline.html <- function(){
    # Function for generating an HTML string
    . <- function(block){
        s <- Stencil()
        s$target()
        eval(block)
        s$untarget()
        s$html()
    }

    checkEquals(
        '<p />',
        .(p())
    )
    
    checkEquals(
        '<p>Hello world</p>',
        .(p('Hello world'))
    )

    checkEquals(
        '<p class="special">Hello world</p>',
        .(p(class='special','Hello world'))
    )
    
    checkEquals(
        '<p>2+2=4</p>',
        .(p('2+2=',2+2))
    )
}
