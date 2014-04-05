#pragma once

#include <stack>
#include <string>

#include <stencila/component.hpp>
#include <stencila/utilities/xml.hpp>

namespace Stencila {
namespace Contexts {

class Context : public Component {
protected:

    /**
     * Method to throw an "unsupported" exception
     */
    void unsupported_(const std::string& method){
        throw Exception("Method \"" + method + "\" not supported by this type of context");
    }

public:

    std::string type(void) const {
        return "context";
    }

    /**
     * @name Rendering methods
     *
     * Methods related to rendering of stencils
     *
     * @{
     */

    /**
     * Execute code within the context
     * 
     * @param code String of code
     */
    Context& execute(const std::string& code){
        unsupported_("execute");
    }
    
    /**
     * Execute a peice of code and return an interactive result
     *
     * This method is used for allowing contexts to be use in a 
     * [read-eval-print loop](http://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop).
     * 
     * @param  code String of code
     * @return      String representation of the result of executing the code
     */
    std::string interact(const std::string& code){
        unsupported_("interact");
    }

    /**
     * Assign an expression to a name.
     * Used by stencil `import` and `include` elements to assign values
     * to the context of the transcluded stencils.
     * 
     * @param name       Name to be assigned
     * @param expression Expression to be assigned to name
     */
    Context& assign(const std::string& name, const std::string& expression){
        unsupported_("assign");
    }

    /**
     * Get a text representation of an expression. 
     * Used by stencil `text` elements e.g. `<span data-text="x">42</span>`
     * 
     * @param  expression Expression to convert to text
     */
    std::string write(const std::string& expression){
        unsupported_("write");
    }

    /**
     * Create an image from the code
     * Used by stencil `image` elements e.g. `<code data-image="png">plot(x,y)</code>`
     * 
     * @param format A support image format e.g. svg, png
     */
    std::string paint(const std::string& format,const std::string& code){
        unsupported_("paint");
    }
 
    /**
     * Test whether an expression is true or false. 
     * Used by stencil `if` elements e.g. `<span data-if="height>10">The height is greater than 10</span>`
     * 
     * @param  expression Expression to evaluate
     */
    bool test(const std::string& expression){
        unsupported_("test");
    }

    /**
     * Mark an expression to be the subject of subsequent `match` queries.
     * Used by stencil `switch` elements e.g. `<p data-switch="x"> X is <span data-match="1">one</span><span data-default>not one</span>.</p>`
     * 
     * @param expression Expression to evaluate
     */
    Context& mark(const std::string& expression){
        unsupported_("mark");
    }

    /**
     * Test whether an expression matches the current subject.
     * Used by stencil `match` elements (placed within `switch` elements)
     * 
     * @param  expression Expression to evaluate
     */
    bool match(const std::string& expression){
        unsupported_("match");
    }

    /**
     * Unmark the current subject expression
     */
    Context& unmark(void){
        unsupported_("unmark");
    }
    
    /**
     * Begin a loop.
     * Used by stencil `for` elements e.g. `<ul data-for="planet:planets"><li data-each data-text="planet" /></ul>`
     * 
     * @param  item  Name given to each item
     * @param  expression Expression giveing an iterable list of items
     */
    bool begin(const std::string& item,const std::string& expression){
        unsupported_("begin");
    }

    /**
     * Steps the current loop to the next item. 
     * Used by stencil `for` elements. See stencil `render`ing methods.
     */
    bool next(void){
        unsupported_("next");
    }

    /**
     * Ends the current loop.
     * Used by stencil `end` elements e.g. `<div data-if="x<-3"><div data-end /></div>`
     */
    bool leave(void){
        unsupported_("leave");
    }

    /**
     * Enter a new namespace. 
     * Used by stencil `with` element e.g. `<div data-with="mydata"><span data-text="sum(a*b)" /></div>`
     *  
     * @param expression Expression that will be the scope of the new context
     */
    Context& enter(const std::string& expression=""){
        unsupported_("enter");
    }

    /**
     * Exit the current namespace
     */
    Context& exit(void){
        unsupported_("exit");
    }

    /**
     * @}
     */

};

}
}
