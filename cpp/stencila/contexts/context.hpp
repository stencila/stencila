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

    /**
     * Execute code within the context
     * 
     * @param code String of code
     */
    void execute(const std::string& code){
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
        return "";
    }

    /**
     * Assign an expression to a name
     * 
     * @param name       Name to be assigned
     * @param expression Expression to be assigned to name
     */
    void assign(const std::string& name, const std::string& expression){
    }

    /**
     * Get a text representation of an expression. 
     * Used by stencil `text` elements e.g. `<span data-text="x">42</span>`
     * 
     * @param  expression Expression to convert to text
     */
    std::string text(const std::string& expression){
        return "";
    }
 
    /**
     * Test whether an expression is logically true or false. 
     * Used by stencil `if` elements e.g. `<span data-if="height>10">The height is greater than 10</span>`
     * 
     * @param  expression Expression to evaluate
     */
    bool test(const std::string& expression){
        return false;
    }

    /**
     * Make an expression the subject of subsequent `match` queries.
     * Used by stencil `switch` elements e.g. `<p data-switch="x"> X is <span data-match="1">one</span><span data-default>not one</span>.</p>`
     * 
     * @param expression Expression to evaluate
     */
    void subject(const std::string& expression){
    }

    /**
     * Test whether an expression matches the current subject.
     * Used by stencil `match` elements (placed within `switch` elements)
     * 
     * @param  expression Expression to evaluate
     */
    bool match(const std::string& expression){
        return false;
    }

    /**
     * End the current subject
     */
    void unsubject(void){
    }
    
    /**
     * Begin a loop.
     * Used by stencil `for` elements e.g. `<ul data-for="planet:planets"><li data-text="planet" /></ul>`
     * 
     * @param  item  Name given to each item
     * @param  expression Expression giveing an iterable list of items
     */
    bool begin(const std::string& item,const std::string& expression){
        return false;
    }

    /**
     * Steps the current loop to the next item. 
     * Used by stencil `for` elements. See stencil `render`ing methods.
     */
    bool next(void){
        return false;
    }

    /**
     * Ends the current loop.
     * Used by stencil `end` elements e.g. `<div data-if="x<-3"><div data-end /></div>`
     */
    bool end(void){
        return false;
    }

    /**
     * Enter a new child context. 
     * Used by stencil `with` element e.g. `<div data-with="mydata"><span data-text="sum(a*b)" /></div>`
     *  
     * @param expression Expression that will be the scope of the new context
     */
    void enter(const std::string& expression=""){
    }

    /**
     * Exit the current child context
     */
    void exit(void){
    }

};

}
}
