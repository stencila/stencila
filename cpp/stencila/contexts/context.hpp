#pragma once

#include <string>

#include <stencila/component.hpp>

namespace Stencila {
namespace Contexts {

template<class Class>
class Context : public Component<Context<Class>> {
public:

    std::string type(void) const {
        return "context";
    }

protected:

    void unsupported(void){
        throw Exception("Not supported by context type: "+static_cast<Class&>(*this).type());
    }

    /**
     * Execute code within the context
     * 
     * @param code String of code
     */
    Context& execute(const std::string& code){
        unsupported();
    }
    
    /**
     * Execute a peice of code and return an interactive result
     *
     * This method is used for allowing context to be use in a 
     * [read-eval-print loop](http://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop).
     * 
     * @param  code String of code
     * @return      String representation of the result of executing the code
     */
    std::string interact(const std::string& code){
        unsupported();
    }

    /**
     * Assign an expression to a name.
     * Used by stencil `import` and `include` elements to assign values
     * to the context of the transcluded stencils.
     * 
     * @param name       Name to be assigned
     * @param expression Expression to be assigned to name
     */
    Context& assign(const std::string& name, const std::string& expression){\
        unsupported();
    }

    /**
     * Get a text representation of an expression. 
     * Used by stencil `text` elements e.g. `<span data-text="x">42</span>`
     * 
     * @param  expression Expression to convert to text
     */
    std::string text(const std::string& expression){
        unsupported();
    }

    /**
     * Create an image from the code
     * Used by stencil `image` elements e.g. `<code data-image="png">plot(x,y)</code>`
     * 
     * @param format A support image format e.g. svg, png
     */
    std::string image(const std::string& format,const std::string& code){
        unsupported();
    }
 
    /**
     * Test whether an expression is logically true or false. 
     * Used by stencil `if` elements e.g. `<span data-if="height>10">The height is greater than 10</span>`
     * 
     * @param  expression Expression to evaluate
     */
    bool test(const std::string& expression){
        unsupported();
    }

    /**
     * Make an expression the subject of subsequent `match` queries.
     * Used by stencil `switch` elements e.g. `<p data-switch="x"> X is <span data-match="1">one</span><span data-default>not one</span>.</p>`
     * 
     * @param expression Expression to evaluate
     */
    Context& subject(const std::string& expression){
        unsupported();
    }

    /**
     * Test whether an expression matches the current subject.
     * Used by stencil `match` elements (placed within `switch` elements)
     * 
     * @param  expression Expression to evaluate
     */
    bool match(const std::string& expression){
        unsupported();
    }

    /**
     * End the current subject
     */
    Context& unsubject(void){
        unsupported();
    }
    
    /**
     * Begin a loop.
     * Used by stencil `for` elements e.g. `<ul data-for="planet:planets"><li data-each data-text="planet" /></ul>`
     * 
     * @param  item  Name given to each item
     * @param  expression Expression giveing an iterable list of items
     */
    bool begin(const std::string& item,const std::string& expression){
        unsupported();
    }

    /**
     * Steps the current loop to the next item. 
     * Used by stencil `for` elements. See stencil `render`ing methods.
     */
    bool next(void){
        unsupported();
    }

    /**
     * Ends the current loop.
     * Used by stencil `end` elements e.g. `<div data-if="x<-3"><div data-end /></div>`
     */
    bool end(void){
        unsupported();
    }

    /**
     * Enter a new child context. 
     * Used by stencil `with` element e.g. `<div data-with="mydata"><span data-text="sum(a*b)" /></div>`
     *  
     * @param expression Expression that will be the scope of the new context
     */
    Context& enter(const std::string& expression=""){
        unsupported();
    }

    /**
     * Exit the current child context
     */
    Context& exit(void){
        unsupported();
    }

};

}
}
