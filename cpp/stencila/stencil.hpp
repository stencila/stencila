#pragma once

#include <stencila/component.hpp>
#include <stencila/context.hpp>
#include <stencila/xml.hpp>

namespace Stencila {

class Stencil : public Component, public Xml::Document {
public:

	typedef Xml::Attribute Attribute;
	typedef Xml::Attributes Attributes;
	typedef Xml::Node Node;
	typedef Xml::Nodes Nodes;

	// Avoid ambiguities by defining which inherited method to use
	// when the base classes (Component & Xml::Document) use the same name
	using Component::path;
	using Component::destroy;

	Stencil(void){
	}

	Stencil(const std::string& from){
		initialise(from);
	}

	~Stencil(void){
		if(context_) delete context_;
		if(outline_) delete outline_;
	}

	/**
	 * @name Input and output
	 *
	 * Methods implemented in `stencil-io.cpp`
	 * 
	 * @{
	 */

	/**
	 * Initialise a stencil
	 * 
	 * @param  from A string indicating how the stencil is initialised
	 */
	Stencil& initialise(const std::string& from);

	/**
	 * Import the stencil content from a file
	 * 
	 * @param  path Filesystem path to file
	 */
	Stencil& import(const std::string& path="");

	/**
	 * Export the stencil content to a file
	 * 
	 * @param  path Filesystem path to file
	 */
	Stencil& export_(const std::string& path="");

	/**
	 * Read the stencil from a directory
	 * 
	 * @param  directory Filesystem path to a directory. 
	 *                   If an empty string (`""`) then the stencil's current path is used.
	 */
	Stencil& read(const std::string& directory="");
	
	/**
	 * Write the stencil to a directory
	 * 
	 * @param  directory Filesystem path to a directory
	 */
	Stencil& write(const std::string& directory="");

	/**
	 * @}
	 */

	/**
	 * @name HTML parsing and generation
	 *
	 * Methods implemented in `stencil-html.cpp`
	 * 
	 * @{
	 */
	
	/**
	 * Get stencil content as HTML
	 */
	std::string html(bool document = false, bool indent = true) const;

	/**
	 * Set stencil content as HTML
	 *
	 * This method parses the supplied HTML, tidying it up in the process, 
	 * and appends the resulting node tree to the stencil's XML tree
	 * 
	 * @param html A HTML string
	 */
	Stencil& html(const std::string& html);

	/**
	 * @}
	 */
	

	/**
	 * @name Cila parsing and generation
	 *
	 * Methods implemented in `stencil-cila.cpp`
	 * 
	 * @{
	 */

	/**
	 * Get stencil content as Cila
	 */
	std::string cila(void) const;

	/**
	 * Get stencil content as Cila written to an output stream
	 *
	 * @param stream Output stream to write to
	 */
	std::ostream& cila(std::ostream& stream) const;

	/**
	 * Set stencil content using Cila
	 * 
	 * @param cila A string of Cila code
	 */
	Stencil& cila(const std::string& cila);

	/**
	 * Set stencil content using Cila read from an input stream
	 *
	 * @param stream Input stream to read from
	 */
	Stencil& cila(std::istream& stream);

	/**
	 * @}
	 */
		
	/**
	 * @name User inputs
	 *
	 * Methods implemented in `stencil-inputs.cpp`
	 * 
	 * @{
	 */

	/**
	 * An `<input>` element (e.g. `<input name="answer" type="number" value="42"></input>`)
	 *
	 * `input` elements are used for setting variables in the context using untrusted
	 * user supplied data. Variables must be of a specified type.
	 * For trusted user content, the analogue of an `<input>` element
	 * is a `set` directive which takes a language expression (which may be any type). 
	 * 
	 * @param node    Node to render
	 * @param context Context to render in
	 */
	struct Input {
		std::string name;
		std::string type;
		std::string value;

		Input(Node node);

		void render(Node node, Context* context);
	};

	/**
	 * Set this stencil's inputs
	 *
	 * The `html` and `cila` methods can be used for changing
	 * a stencil's content from a trusted source. In contrast, 
	 * this method is intended for inputs from an untrusted user.
	 * It maps the supplied `name:value` pairs into `<input>` elements
	 * (which may, or may not, be within `par` directives).
	 * 
	 * @param inputs A map of `name:value` pairs of inputs
	 */
	Stencil& inputs(const std::map<std::string,std::string>& inputs);

	/**
	 * @}
	 */

	/**
	 * @name Attributes
	 *
	 * Methods for obtaining or setting attributes of the stencil.
	 *
	 * Methods implemented in `stencil-attrs.cpp`
	 * 
	 * @{
	 */

	/**
	 * Get this stencil's title
	 */
	std::string title(void) const;

	/**
	 * Get this stencil's description
	 */
	std::string description(void) const;

	/**
	 * Get this stencil's keywords
	 */
	std::vector<std::string> keywords(void) const;

	/**
	 * Get this stencil's authors
	 */
	std::vector<std::string> authors(void) const;
	
	/**
	 * Get the list of rendering contexts that are compatible with this stencil.
	 *
	 * Context compatability will be determined by the expressions used in 
	 * stencil directives like `data-with`,`data-text` etc. Some expressions
	 * will be able to be used in multiple contexts.
	 */
	std::vector<std::string> contexts(void) const;


	/**
	 * Get this stencil's theme
	 */
	std::string theme(void) const;

	/**
	 * @}
	 */
	
	/**
	 * @name Directives
	 *
	 * Methods implemented in `stencil-directives.cpp`
	 * 
	 * @{
	 */
	
	/*
	 * List of stencil directive attributes
	 */
	static const std::vector<std::string> directives;

	/*
	 * List of stencil flag attributes
	 */
	static const std::vector<std::string> flags;
	
	/**
	 * Is the attribute a stencil directive?
	 */
	static bool directive(const std::string& attr);

	/**
	 * Is the attribute a stencil flag?
	 */
	static bool flag(const std::string& attr);

	struct Directive {
		typedef std::string Name;
		typedef std::string Expression;
		typedef bool Flag;
	};

	struct DirectiveException : Exception {
		std::string type;
		std::string data;

		DirectiveException(const std::string& type, const std::string& data):
			type(type),data(data){
		}
	};

	/**
	 * Render an error onto a directive node.
	 * 
	 * A function for providing consistent error reporting from
	 * within rendering functions.
	 * 
	 * @param node    Node where error occurred
	 * @param type    Type of error, usually prefixed with directive type e.g. `for-syntax`
	 * @param data    Data associated with the error which may be useful for resolving it
	 */
	static void error(Node node, const std::string& type, const std::string& data);

	/**
	 * An execute (`exec`) directive (e.g. `<pre data-exec="r,py">`)
	 *
	 * The text of the element is executed in the context if the context's type
	 * is listed in the `data-exec` attribute. If the context's type is not listed
	 * then the element will not be rendered (i.e. will not be executed). 
	 * 
	 * This behaviour allows for polyglot stencils which have `exec` directives that
	 * are either polyglot (valid in more than one languages) or monoglot (valid in only one language)
	 * as required by similarities/differences in the language syntax e.g.
	 *
	 *    <pre data-exec="r,py">
	 *        m = 1
	 *        c = 299792458
	 *    </pre>
	 * 
	 *    <pre data-exec="r"> e = m * c^2 </pre>
	 *    <pre data-exec="py"> e = m * pow(c,2) </pre>
	 */
	struct Execute : Directive {
		bool valid;
		std::vector<Name> contexts;
		Expression format;
		Expression width;
		Expression height;
		Expression units;
		std::string size;
		Flag constant = false;
		Flag show = false;

		Execute(const std::string& attribute);
		Execute(Node node);
		void parse(const std::string& attribute);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A parameter (`par`) directive
	 */
	struct Parameter : Directive {
		Name name;
		Name type;
		Expression value;

		Parameter(const std::string& attribute);
		Parameter(Node node);
		void parse(const std::string& attribute);
		void render(Node node, Context* context);
	};

	/**
	 * Get this stencil's parameters
	 */
	std::vector<Parameter> pars(void) const;

	/**
	 * A set (`set`) directive (e.g. `<span data-set="answer to 42"></span>`)
	 *
	 * The expression in the `data-set` attribute is assigned to the name in the context.
	 */
	struct Set : Directive {
		Name name;
		Expression value;

		Set(const std::string& attribute);
		Set(Node node);
		void parse(const std::string& attribute);
		void render(Node node, Context* context);
	};

	/**
	 * An `include` directive (e.g. `<div data-include="stats/t-test select #macros #text #simple-paragraph" />` )
	 */
	struct Include : Directive {
		Expression address;
		Expression select;

		Include(const std::string& attribute);
		Include(Node node);
		void parse(const std::string& attribute);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * @}
	 */

	/**
	 * @name Rendering
	 *
	 * Methods implemented in `stencil-render.cpp`
	 * 
	 * @{
	 */
	
	/**
	 * Attach a context to this stencil
	 *
	 * @param context Context for rendering
	 */
	Stencil& attach(Context* context);

	/**
	 * Detach the stencil's current context
	 *
	 * The method `delete`s the context.
	 */
	Stencil& detach(void);

	/**
	 * Get details on this stencil's current context
	 *
	 * The method `delete`s the context.
	 */
	std::string context(void) const;

	/**
	 * Render a `write` directive (e.g. `<span data-write="result"></span>`)
	 *
	 * The expression in the `data-write` attribute is converted to a 
	 * character string by the context and used as the element's text.
	 * If the element has a `data-off="true"` attribute then the element will not
	 * be rendered and its text will remain unchanged.
	 */
	void render_write(Node node, Context* context);

	/**
	 * Render a `with` element (e.g. `<div data-with="sales"><span data-text="sum(quantity*price)" /></div>` )
	 *
	 * The expression in the `data-with` attribute is evaluated and made the subject of a new context frame.
	 * All child nodes are rendered within the new frame. The frame is then exited.
	 */
	void render_with(Node node, Context* context);

	/**
	 * Render a `if` element (e.g. `<div data-if="answer==42">...</div>` )
	 *
	 * The expression in the `data-if` attribute is evaluated in the context.
	 */
	void render_if(Node node, Context* context);

	/**
	 * Render a `switch` element
	 *
	 * The first `case` element (i.e. having a `data-case` attribute) that matches
	 * the `switch` expression is activated. All other `case` and `default` elements
	 * are deactivated. If none of the `case` elements matches then any `default` elements are activated.
	 */
	void render_switch(Node node, Context* context);

	/**
	 * Render a `for` directive e.g. `<ul data-for="planet in planets"><li data-text="planet" /></ul>`
	 *
	 * A `for` directive has a `data-for` attribute which specifies the variable name given to each item and 
	 * an expression providing the items to iterate over e.g. `planet in planets`
	 *
	 * The first child element is rendered for each item and given a `data-index="<index>"`
	 * attribute where `<index>` is the 0-based index for the item. If the `for` element has already been rendered and
	 * already has a child with a corresponding `data-index` attribute then that is used, otherwise a new child is appended.
	 * This behaviour allows for a user to `data-lock` an child in a `for` element and not have it lost. 
	 * Any child elements with a `data-index` greater than the number of items is removed unless it has a 
	 * descendent with a `data-lock` attribute in which case it is retained but marked with a `data-extra` attribute.
	 */
	void render_for(Node node, Context* context);

	/**
	 * Render the children of an HTML element
	 * 
	 * @param node    Node to render
	 * @param context Context to render in
	 */
	void render_children(Node node, Context* context);

	/**
	 * Update and render a intra-stencil dependecy hash on the node
	 */
	bool render_hash(Node node);

	/**
	 * Initialise the rendering of a stencil
	 *
	 * @param node    Node to render
	 * @param context Context to render in
	 */
	void render_initialise(Node node, Context* context);

	/**
	 * Finalise the rendering of a stencil (e.g. adding missing elements)
	 *
	 * @param node    Node to render
	 * @param context Context to render in
	 */
	void render_finalise(Node node, Context* context);

	/**
	 * Render a HTML element
	 * 
	 * @param node    Node to render
	 * @param context Context to render in
	 */
	void render(Node node, Context* context);

	/**
	 * Render this stencil within a context
	 * and attach the context.
	 *
	 * @param context Context for rendering
	 */
	Stencil& render(Context* context);

	/**
	 * Render this stencil in a new context
	 * 
	 * @param  type Type of context (e.g. "r", "py")
	 */
	Stencil& render(const std::string& type);

	/**
	 * Render this stencil, using the currenly attached context, or 
	 * creating a new context if necessary
	 */
	Stencil& render(void);

	/**
	 * Remove attributes and element added to this stencil during
	 * previous renderings
	 */
	Stencil& strip(void);

	/**
	 * Strip and then render this stencil
	 */
	Stencil& restart(void);

	/**
	 * @}
	 */
	
	/**
	 * @name Serving
	 *
	 * Methods for serving a stencil over a nework.
	 * Overrides of `Component` methods as required.
	 *
	 * Methods implemented in `stencil-serve.cpp`
	 * 
	 * @{
	 */

	/**
	 * Serve this stencil
	 */
	std::string serve(void);

	/**
	 * View this stencil
	 */
	void view(void);

	/**
	 * Interact with this stencil
	 */
	std::string interact(const std::string& code);

	/**
	 * Execute a call on this stencil
	 * 
	 * @param  call A `Call` object
	 */
	std::string call(const Call& call);

	/**
	 * Generate a web page for a stencil
	 */
	static std::string page(const Component* component);

	/**
	 * Execute a call on a stencil
	 */
	static std::string call(Component* component, const Call& call);

	/**
	 * @}
	 */

	/**
	 * @name Sanitization
	 * @{
	 */
	
	/**
	 * List of allowed stencil element names?
	 */
	static const std::vector<std::string> tags;

	/**
	 * Is the element tag name an allowed stencil element?
	 */
	static bool tag(const std::string& name);

	/**
	 * Sanitize the stencil to remove potenitally malicious elements
	 * and attributes
	 */
	Stencil& sanitize(void);

	/**
	 * @}
	 */

	/**
	 * Commit changes to this stencil
	 * 
	 * @param  message A commit message
	 */
	Stencil& commit(const std::string& message=""){
		// Save the stencil..
		write();
		///...then commit it
		Component::commit(message);
		return *this;
	}

private:

	/**
	 * The current rendering context for this stencil
	 */
	Context* context_ = nullptr;

	/**
	 * A record of the number of elements of particular types within
	 * this stencil
	 */
	std::map<std::string,unsigned int> counts_;

	/**
	 * A hash used to track intra-stencil dependencies
	 */
	std::string hash_;

	/**
	 * Outlining, including section numbering and table of content, handled
	 * by `Outline` struct
	 */
	struct Outline;
	Outline* outline_ = nullptr;
};

}
