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

	Stencil(void);

	Stencil(const std::string& from);

	~Stencil(void);

	/**
	 * Get the component type
	 */
	static Type type(void){
		return StencilType;
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
	 * @param  path Filesystem path to a directory. 
	 *              If an empty string (`""`) then the stencil's current path is used.
	 */
	Stencil& read(const std::string& path="");
	
	/**
	 * Write the stencil to a directory
	 * 
	 * @param  path Filesystem path to a directory
	 */
	Stencil& write(const std::string& path="");

	/**
	 * @}
	 */
	
	/**
	 * @name XML parsing and generation
	 *
	 * Methods implemented in `stencil-xml.cpp`
	 * 
	 * @{
	 */
	
	/**
	 * Get stencil content as XML
	 */
	std::string xml(void) const;

	/**
	 * Set stencil content as XML
	 * 
	 * @param xml A XML string
	 */
	Stencil& xml(const std::string& xml);

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
	 * Set stencil content using Cila
	 * 
	 * @param cila A string of Cila code
	 */
	Stencil& cila(const std::string& cila);

	/**
	 * @}
	 */


	/**
	 * @name Conversion to/from other formats
	 *
	 * Methods implemented in `stencil-conversion.cpp`
	 *
	 * Most of these methods have a `direction` parameter; however only one
	 * direction may be currently implemented.
	 * 
	 * @{
	 */
	
	/**
	 * Convert to/from a Microsoft Word document
	 * 
	 * @param  direction  The direction of conversion; one of "to" or "from"
	 * @param  path       Path of the DOCX file to read from / write to (dependending on direction)
	 */
	Stencil& docx(const std::string& direction, const std::string& path);

	/**
	 * Convert to/from a Markdown document
	 * 
	 * @param  direction  The direction of conversion; one of "to" or "from"
	 * @param  path       Path of the Markdown to read from / write to (dependending on direction)
	 */
	Stencil& markdown(const std::string& direction, const std::string& path);

	/**
	 * Convert to/from a Portable Document Format (PDF) document
	 * 
	 * @param  direction    The direction of conversion; one of "to" or "from"
	 * @param  path         Path of PDF file to create
	 * @param  format       One of "A3", "A4", "A5", "legal", "letter", "tabloid"
	 * @param  orientation  Either "portrait" or "landscape"
	 * @param  margin       Width of margins e.g. "2cm" . Allowed units are "mm", "cm", "in", "px"
	 * 
	 */
	Stencil& pdf(
		const std::string& direction, 
		const std::string& path,
		const std::string& format = "A4",
		const std::string& orientation = "portrait",
		const std::string& margin = "1cm"
	);

	/**
	 * Compile this stencil
	 *
	 * Render this stencil and export it as HTML to `stencil.html` and
	 * a create a preview image as `preview.png`
	 */
	Stencil& compile(void);
	
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

		Input(void);
		Input(Node node);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
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
	 *
	 * @param versioned Should the theme be returned with a version (if specified)?
	 */
	std::string theme(bool versioned=true) const;

	/**
	 * Is this stencil closed?
	 */
	bool closed(void) const;

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

	/**
	 * Remove attributes and elements added to a HTML node,
	 * and its children, during rendering
	 */
	static void strip(Node node);

	/**
	 * Remove attributes and elements added to this stencil
	 * during rendering
	 */
	Stencil& strip(void);

	/**
	 * Remove all directives and flags from a HTML node and its children
	 */
	static void crush(Node node);

	/**
	 * Remove all directives and flags from this stencil
	 */
	Stencil& crush(void);

	/**
	 * Create a hash of a string key. Used to keep track
	 * of intra-stencil depenedencies
	 */
	std::string hash(Node node, bool attrs=true, bool text=true);


	struct Directive {
		typedef std::string Name;

		typedef std::string Expression;

		struct Evaluatable {
			bool eval = false;
			std::string expr;
			std::string value;

			std::string evaluate(Context* context) {
				if(eval and expr.length()) value = context->write(expr);
				else value = expr;
				return value;
			}

		};

		typedef bool Flag;
	};

	struct DirectiveException : Exception {
		std::string type;
		std::string data;

		DirectiveException(const std::string& type, const std::string& data=""):
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
	static void error(Node node, const std::string& type, const std::string& data="");

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
		Evaluatable format;
		Evaluatable width;
		Evaluatable height;
		Evaluatable units;
		Evaluatable size;
		Flag constant = false;
		Flag show = false;

		Execute(void);
		Execute(const std::string& attribute);
		Execute(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * Get a list of this stencil's execute directives
	 */
	std::vector<Execute> execs(void) const;

	/**
	 * A `when` directive (e.g. `<div data-when="py,cpp"><span data-text="point.x" /></div>` )
	 *
	 * Used to restrict the rendering of sections of a stencil to only some context types.
	 * This is useful when evaluated expressions are specific to a particular context type.
	 * If the current context does not accept one of the labels in the comma separated list the
	 * section is turned off.
	 */
	struct When : Directive {
		std::vector<Name> contexts;

		void parse(const std::string& attribute);
		void scan(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * An `attr` directive (e.g. `<span data-attr="src image+'.png'"></span>`)
	 *
	 * Adds or sets an attribute on the parent element
	 */
	struct Attr : Directive {
		Name name;
		Expression expression;

		Attr(void);
		Attr(const std::string& attribute);
		Attr(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `text` directive (e.g. `<span data-text="result"></span>`)
	 *
	 * The expression in the `data-text` attribute is converted to a 
	 * character string by the context and used as the element's text.
	 * If the element has a `data-off="true"` attribute then the element will not
	 * be rendered and its text will remain unchanged.
	 */
	struct Text : Directive {
		Expression expression;

		Text(void);
		Text(const std::string& attribute);
		Text(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `with` directive (e.g. `<div data-with="sales"><span data-text="sum(quantity*price)" /></div>` )
	 *
	 * The expression in the `data-with` attribute is evaluated and made the subject of a new context namespace.
	 * All child nodes are rendered within the new namespace. The namespace is then exited.
	 */
	struct With : Directive {
		Expression expression;

		With(void);
		With(const std::string& attribute);
		With(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `if` directive (e.g. `<div data-if="answer==42">...</div>` )
	 *
	 * The expression in the `data-if` attribute is evaluated in the context.
	 */
	struct If : Directive {
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `switch` directive
	 *
	 * The first `case` element (i.e. having a `data-case` attribute) that matches
	 * the `switch` expression is activated. All other `case` and `default` elements
	 * are deactivated. If none of the `case` elements matches then any `default` elements are activated.
	 */
	struct Switch : Directive {
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `for` directive e.g. `<ul data-for="planet in planets"><li data-text="planet" /></ul>`
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
	struct For : Directive {
		Name item;
		Expression items;

		For();
		For(const std::string& attribute);
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

		Parameter(void);
		Parameter(const std::string& attribute);
		Parameter(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * Get a list of this stencil's parameter directives
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

		Set(void);
		Set(const std::string& attribute);
		Set(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * An `include` directive (e.g. `<div data-include="stats/t-test select #macros #text #simple-paragraph" />` )
	 */
	struct Include : Directive {
		Evaluatable address;
		Evaluatable select;
		/**
		 * By default the included content is `crush`ed thereby removing all stencil directives
		 * and declarations. When `complete = true` no crushing is done. This can be useful when
		 * debugging an included stencil to discover why it did not render as expected.
		 */
		Flag complete = false;

		Flag names = false;

		Include(void);
		Include(const std::string& attribute);
		Include(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `macro` directive (e.g. `<div data-macro="my-macro" />` )
	 */
	struct Macro : Directive {
		Name name;
		
		Macro(void);
		Macro(const std::string& attribute);
		Macro(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
		void render(Stencil& stencil, Node node, Context* context);
	};

	/**
	 * A `create` directive (e.g. `<div data-create="x from core/stencils/table" />` )
	 */
	struct Create : Directive {
		Name name;
		Evaluatable address;
		Evaluatable select;
		
		Create(void);
		Create(const std::string& attribute);
		Create(Node node);
		void parse(const std::string& attribute);
		void parse(Node node);
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
	 * Create an alias to a node
	 *
	 * Aliases are usually only produced by `alias` 
	 * directives e.g.
	 *  
	 *      `<div data-alias="z is x/y"></div>` 
	 *      
	 * results in a alias "z" which points to the node with address x/y
	 * 
	 * @param  alias [description]
	 * @param  node  [description]
	 */
	Stencil& alias(const std::string& alias, Node node);

	/**
	 * Retreive the node associated with an alias 
	 *
	 * Called by `include` directives to resolve a name to a node.
	 * If no matching alias is found then name is assumed to be
	 * a stencil address.
	 */
	Node alias(const std::string& alias);

	/**
	 * Remove an alias
	 * 
	 * @param  alias [description]
	 */
	Stencil& unalias(const std::string& alias);

	/**
	 * Remove all aliases
	 */
	Stencil& unalias(void);

	/**
	 * Create a link between a context namespace and a node
	 * 
	 * Links are created by the `snip` directive. e.g.
	 * 
	 *    `<div data-snip="m from p/q"></div>`
	 *
	 * creates a namespace in the stencil's context called `m`. This namespace 
	 * has a unique link string (e.g. "af73cd") stored in the special variable
	 * `__link__`. By calling this method, that link string is used to link the node `p/q` to the
	 * namespace `m`.
	 */
	Stencil& link(const std::string& link, Node node);

	/**
	 * Retreive the node associated with a link
	 * 
	 * Used when a snippet is pasted e.g.
	 * 
	 *    `<div data-paste="m #scatter-plot"></div>` 
	 *
	 * The variable "m" is retreived from the context, it's special variable
	 * `__link__` is obtained and is then used, by calling this method,
	 * to retreive the corresponding node so that it can be rendered, within the
	 * namespace "m" and inserted into the stencil.
	 */
	Node link(const std::string& link);

	/**
	 * Remove a link
	 * 
	 * @param  link A link string
	 */
	Stencil& unlink(const std::string& link);

	/**
	 * Remove all links
	 */
	Stencil& unlink(void);

	/**
	 * Render the children of an HTML element
	 * 
	 * @param node    Node to render
	 * @param context Context to render in
	 */
	void render_children(Node node, Context* context);

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
	 * Strip and render this stencil
	 */
	Stencil& refresh(void);

	/**
	 * Read (in case of local file changes), strip and render this stencil
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
	Stencil& view(void);

	/**
	 * Create a preview of this stencil
	 */
	Stencil& preview(const std::string& path);

	/**
	 * Interact with this stencil
	 */
	std::string interact(const std::string& code);

	/**
	 * Generate a web page for a stencil
	 *
	 * @param  component  A pointer to a stencil
	 */
	static std::string page(const Component* component);

	/**
	 * Generate a web page for this stencil
	 */
	std::string page(void) const;

	/**
	 * Execute a call on a stencil
	 *
	 * @param  component  A pointer to a stencil
	 */
	static std::string call(Component* component, const Call& call);

	/**
	 * Execute a call on this stencil
	 * 
	 * @param  call  A `Call` object
	 */
	std::string call(const Call& call);

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
	 * Outlining, including section numbering and table of contents, is handled
	 * by an `Outline` struct.
	 */
	struct Outline {
		bool on = false;
		Node list;
		uint index;
		std::vector<int> path;
	} outline_;

	std::map<std::string,std::pair<Node,bool>> aliases_;

};

}
