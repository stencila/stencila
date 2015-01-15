#pragma once

#include <boost/regex.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/lexical_cast.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

class CilaParser {
public:
	typedef Stencil::Node Node;

	/**
	 * Alternative parsing states
	 */
	enum State {
		/**
		 * Start of line state
		 */
		sol,

		/**
		 * Looking for element
		 *
		 * In this state can move across into `attrs`
		 */
		elem,

		/**
		 * Looking for element attributes
		 *
		 * In this state the parser is looking for HTML element attribute
		 * syntax (e.g. `[id="an-id"]`, `#an-id`, `.a-class`) including directives (e.g. `write x`) 
		 * and ignoring whitepace. If no attribute is found then moves across to `text` state.
		 */
		attrs,

		/**
		 * Text including inlines, shortcuts and embedded elements
		 */
		text,

		/**
		 * Within an emphasis section (e.g `_this text is emphasised_`)
		 */
		empha,

		/**
		 * Within a strong section (e.g `*this text is strong*`)
		 */
		strong,

		/**
		 * Within an interpolation section (e.g ``answer``)
		 */
		interp,		

		/**
		 * Within a code section (e.g `answer = 42`)
		 */
		code,

		/**
		 * Within an AsciiMath section (e.g `|e = mc^2|`)
		 */
		asciimath,

		/**
		 * Within a TeX/LaTeX section (e.g `\(e = mc^2\)`)
		 */
		tex,

		/**
		 * Within an `exec` directive
		 */
		exec
	};

	/**
	 * Current state
	 */
	State state;

	/**
	 * State stack.
	 *
	 * Allows for nesting of parsing states. For example, `strong` within `empha`.
	 */
	std::deque<State> states;

	/**
	 * Beggining of input
	 */
	std::string::const_iterator begin;

	/**
	 * End of input
	 */
	std::string::const_iterator end;

	/**
	 * Current regex match
	 *
	 * Used to obtain results from the `is` method
	 */
	boost::smatch match;

	/**
	 * Stencil being parse into
	 */
	Stencil stencil;

	/**
	 * Current indentation. Used for keeping track
	 * of parent-child relationships
	 */
	std::string indent;

	/**
	 * Flag for if indentation has been added to.
	 * Need for "simulated" indentation
	 */
	bool indent_added;

	/**
	 * Current HTML node
	 */
	Node node;

	/**
	 * Stack of nodes for enter/exit
	 */
	struct Element {
		std::string indent;
		Node node;
	};
	std::deque<Element> nodes;

	/**
	 * Buffer of characters to be added as HTML text
	 */
	std::string buffer;

	/**
	 * Flag for orphaned element attributes
	 */
	bool tag_needed;
	bool para_needed;

	/**
	 * Get string representation of a state for debugging
	 */
	std::string state_name(State state) const {
		switch(state){
			#define CASE(STATE) case STATE: return #STATE;
			CASE(sol)
			CASE(elem)
			CASE(attrs)
			CASE(text)
			CASE(empha)
			CASE(strong)
			CASE(interp)
			CASE(code)
			CASE(asciimath)
			CASE(tex)
			CASE(exec)
			#undef CASE
			default: return "";
		}
	}

	/**
	 * Push into a parsing state
	 */
	void push(State to){
		states.push_back(to);
		state = to;
	}

	/**
	 * Pop out of a parsing state
	 */
	void pop(void){
		if(states.size()<2){
			throw std::runtime_error("Too few states to pop: "+boost::lexical_cast<std::string>(states.size()));
		}
		states.pop_back();
		state = states.back();
	}

	/**
	 * Move across into another parsing state
	 */
	void across(State to){
		if(states.size()>0) states.pop_back();
		states.push_back(to);
		state = to;
	}

	/**
	 * Add a character to buffer
	 */
	void add(char cha){
		buffer += cha;
	}

	/**
	 * Add characters to buffer
	 */
	void add(const std::string& chars){
		buffer += chars;
	}

	/**
	 * Add a character to buffer from input
	 */
	void add(void){
		if(begin!=end){
			buffer.insert(buffer.end(),begin,begin+1);
			begin += 1;
		}
	}

	/**
	 * Flush the buffer to the current HTML elment
	 * as a text node
	 */
	Node flush(void){
		Node text;
		if(buffer.length()) text = node.append_text(buffer);
		buffer = "";
		return text;
	}

	/**
	 * Enter a HTML element
	 *
	 * When using this method you are responsible for calling `flush()` first!
	 * 
	 * @param elem Element to enter
	 */
	void enter(Node elem){
		node = elem;
		nodes.push_back({indent,node});
		tag_needed = false;
		para_needed = false;
	}

	/**
	 * Enter a HTML element with given tag name
	 */
	void enter(const std::string& name){
		flush();
		node = node.append(name);
		nodes.push_back({indent,node});
		tag_needed = false;
		para_needed = false;
	}

	/**
	 * Exit a HTML element
	 */
	void exit(){
		flush();

		if(nodes.size()) nodes.pop_back();
		if(nodes.size()) node = nodes.back().node;
		else node = stencil;
	}

	/**
	 * Enter an element and push into a state
	 */
	void enter_push(const std::string& name,State to){
		enter(name);
		push(to);
	}

	/**
	 * Enter an element and move across into a state
	 */
	void enter_across(const std::string& name,State to){
		enter(name);
		across(to);
	}

	/**
	 * Exit an element and pop out of a state
	 */
	void exit_pop(){
		exit();
		pop();
	}

	/**
	 * Check for a regular expression match at the start 
	 * of input buffer
	 * 
	 * @param  regex Regex to find
	 * @return       Was the regex matched?
	 */
	bool is(const boost::regex& regex){
		//match_continuous specifies that the expression must match a sub-sequence that begins at first.
		bool found = boost::regex_search(begin, end, match, regex, boost::regex_constants::match_continuous);
		if(found) begin += match.position() + match.length();
		return found;
	}


#if defined(STENCILA_CILA_PARSER_TRACE)

	// Tracing of state changes used in debugging

	struct Trace {
		State state;
		int states = -1;
		int nodes = -1;
		std::string begin;
		std::string regex = "<?>";
		std::string match = "<?>";
	};
	std::vector<Trace> traces;

	void trace_begin(void){
		traces.clear();
	}

	void trace_new(void){
		Trace current;
		current.state = state;
		current.states = states.size();
		current.nodes = nodes.size();
		current.begin = *begin;
		boost::replace_all(current.begin,"\t","\\t");
		boost::replace_all(current.begin,"\n","\\n");
		boost::replace_all(current.begin," ","\\s");
		traces.push_back(current);
	}

	void trace(const char* regex){
		Trace& current = traces[traces.size()-1];
		current.regex = regex;
		std::string str = "<none>";
		if(not match.empty()){
			try {
				str = match.str();
			}
			catch(const std::logic_error&){
			}
			boost::replace_all(str,"\t","\\t");
			boost::replace_all(str,"\n","\\n");
			boost::replace_all(str," ","\\s");
		}
		current.match = str;
	}

	void trace_show(void) const {
		std::cout<<"-------------------Trace--------------------------------\n";
		std::cout<<"state\tstates\tnodes\tbegin\tregex\t\tmatch\n";
		std::cout<<"--------------------------------------------------------\n";
		for(auto item : traces){
			std::cout<<state_name(item.state)<<"\t"<<item.states<<"\t"<<item.nodes<<"\t"<<item.begin<<"\t"
					<<item.regex<<"\t\t"<<item.match<<"\n";
		}
		std::cout<<"--------------------------------------------------------\n";
		
	}

#else

	// When tracing is off these methods do nothing
	void trace_begin(void){}
	void trace_new(void){}
	void trace(const char* regex){}
	void trace_show(void) const {}

#endif

	/**
	 * Parse a string of Cila
	 * 
	 * @param cila Um, cila to parse
	 */
	CilaParser& parse(const std::string& cila){
		// Initialise members...
		// ... input
		begin = cila.cbegin();
		end = cila.cend();
		// ... states
		states.clear();
		states.push_back(sol);
		state = states.back();
		// ... stencil
		stencil.clear();
		// ... nodes
		nodes.clear();
		nodes.push_back({"",stencil});
		node = nodes.back().node;

		tag_needed = false;
		para_needed = false;

		// Define regular expressions
		static const boost::regex
			indentation("[ \\t]*"),

			exec_open("((r|py)( +[^\\n]+)?)\\n"),

			tag("("\
				"section|nav|article|aside|address|h1|h2|h3|h4|h5|h6|p|hr|pre|blockquote|ol|ul|li|dl|dt|dd|" \
				"figure|figcaption|div|a|em|strong|small|s|cite|q|dfn|abbr|data|time|code|var|samp|kbd|sub|sup|i|b|u|mark|ruby|" \
				"rt|rp|bdi|bdo|span|br|wbr|ins|del|table|caption|colgroup|col|tbody|thead|tfoot|tr|td|th" \
			")\\b"),

			section(">\\s*([ \\w-]+)"),
			ul_item("-\\s*"),
			ol_item("\\d+\\.\\s*"),
			li_shortcut("-|(\\d+\\.)\\s*"),

			attr("([\\w-]+)=([^ ]+)\\b"),
			id("#([\\w-]+)\\b"),
			clas("\\.([\\w-]+)\\b"),
			directive_no_arg("else|default\\b"),
			directive_arg("(write|with|if|elif|switch|case) +([^\\n\\}]+)"),
			spaces(" +"),

			underscore("_"),
			asterisk("\\*"),

			backtick_backtick("``"),

			backtick_escaped("\\\\`"),
			backtick("`"),

			pipe_escaped("\\\\\\|"),
			pipe("\\|"),

			tex_open("\\\\\\("),
			tex_close("\\\\\\)"),

			link("(\\[)([^\\]]*)(\\]\\()([^\\)]+)(\\))"),
			autolink("\\bhttp(s)?://[^ ]+\\b"),

			curly_open("\\{"),
			curly_close("\\}"),

			blankline("[ \\t]*\\n"),
			endline("\\n")
		;

		trace_begin();
		while(begin!=end){
			trace_new();

			if(state==sol){
				if(not boost::regex_search(begin, end, blankline, boost::regex_constants::match_continuous)){
					// Get indentation
					is(indentation);
					indent = match.str();
					// Peek ahead to see if this is a `ul_item` or `ol_item`; for these
					// it is necessary to "simulate" further indentation to ensure correct
					// parent child relationships
					if(boost::regex_search(begin, end, li_shortcut, boost::regex_constants::match_continuous)){
						indent_added = true;
						indent += "\t";
					} else {
						indent_added = false;
					}
					// Exit nodes until a node with lower indentation is reached
					// which then becomes the current node to which others get appended
					while(nodes.size()>1 and (
						nodes.back().indent=="none" or indent.size()<=nodes.back().indent.size()
					)) exit();
				}

				if(is(exec_open)){
					trace("exec");
					// A execute directive should only begin at the 
					// start of a line
					// Enter `<pre>` element and move across to `exec` state;
					enter_across("pre",exec);
					node.attr("data-exec",match[1].str());
				}
				else if(is(blankline)){
					trace("blank");
					para_needed = true;
				}
				else {
					trace("other");
					// Move across into elem state
					across(elem);
				}
			}
			else if(state==elem){
				// Local lambda for entering a list if necessary
				auto enter_list_if_needed = [this](const std::string& name){
					if(node.name()!=name){
						if(indent_added) indent.pop_back();
						enter(name);
						if(indent_added) indent.push_back('\t');
						indent_added = false;
					}
				};
				// Attempt to match...
				if(is(tag)){
					trace("tag");
					// Enter new element and move to `attrs` state to 
					// start looking for attributes
					enter_across(match.str(),attrs);
				}
				else if(is(section)){
					trace("section");
					// Enter `<section>` move into `elem` state to allow
					// for any further attributes
					flush();
					auto id = match[1].str();
					boost::to_lower(id);
					boost::replace_all(id," ","-");
					auto section = node.append("section").attr("id",id);
					auto title = match[1].str();
					auto h1 = section.append("h1").text(title);
					enter(section);
					across(elem);
				}
				else if(is(ul_item)){
					trace("ul_item");
					// Enter `<ul>` if necessary, enter `<li>` and move into `text` state
					enter_list_if_needed("ul");
					enter_across("li",text);
				}
				else if(is(ol_item)){
					trace("ol_item");
					// Enter `<ol>` if necessary, enter `<li>` and move into `text` state
					enter_list_if_needed("ol");
					enter_across("li",text);
				}
				else{
					trace("none");
					// Indicate that a new element is required
					// for any subsequent attributes
					tag_needed = true;
					// Move across to `attrs` state to look for any attributes
					across(attrs);
				}
			}
			else if(state==attrs){
				// Local lambda for entering a new element if needed
				auto enter_elem_if_needed = [this](const std::string& name="div"){
					if(tag_needed) enter(name);
				};
				// Attempt to match...
				if(is(attr)){
					trace("attr");
					// Enter new element it necessary and create attribute;
					// keep on looking for more attributes
					enter_elem_if_needed();
					node.attr(match[1].str(),match[2].str());
				}
				else if(is(id)){
					trace("id");
					// Enter new element it necessary and create id attribute;
					// keep on looking for more attributes
					enter_elem_if_needed();
					node.attr("id",match[1].str());
				}
				else if(is(clas)){
					trace("clas");
					// Enter new element it necessary and create class attribute;
					// keep on looking for more attributes
					enter_elem_if_needed();
					node.attr("class",match[1].str());
				}
				else if(is(directive_no_arg)){
					trace("directive_no_arg");
					// Enter new element it necessary and create directive attribute;
					// move across to `elem` state (i.e no attributes or text to follow)
					enter_elem_if_needed();
					node.attr("data-"+match.str(),"true");
					across(elem);
				}
				else if(is(directive_arg)){
					trace("directive_arg");
					// Enter new element it necessary and create directive attribute;
					// type of element depends on which directive;
					// move across to `elem` state (i.e no attributes or text to follow)
					auto directive = match[1].str();
					if(directive=="write") enter_elem_if_needed("span");
					else enter_elem_if_needed();
					node.attr("data-"+directive,match[2].str());
					across(elem);
				}
				else if(is(spaces)){
					trace("spaces");
					// Ignore spaces and keep on looking for attributes
				}
				else {
					trace("none");
					// If no match move across to `text` state to look 
					// for plain text, shortcuts and embedded elements
					across(text);
				}
			}
			else if(state==text){
				// Enter a new paragraph if necessary
				if(para_needed) enter("p");
				// Any elements that are `enter()`ed from here on
				// will be inlines so set indent to none.
				indent = "none";
				// Attempt to match...
				if(is(curly_open)){
					trace("curly_open");
					// Push into `elem` state
					push(elem);
				}
				else if(is(curly_close)){
					trace("curly_close");
					// Exit from current element and pop out of `text` state
					exit_pop();
				}
				else if(is(underscore)){
					trace("underscore");
					// Enter `<em>` and push into `empha` state
					enter_push("em",empha);
				}
				else if(is(asterisk)){
					trace("asterisk");
					// Enter `<strong>` and push into `strong` state
					enter_push("strong",strong);
				}
				else if(is(backtick_escaped)){
					trace("backtick_escaped");
					// Replace with backtick
					add('`');
				}
				else if(is(backtick_backtick)){
					trace("backtick_backtick");
					// Enter a <span> and push into `interp` state
					enter_push("span",interp);
				}
				else if(is(backtick)){
					trace("backtick");
					// Enter `<code>` and push into `code` state
					enter_push("code",code);
				}
				else if(is(pipe_escaped)){
					trace("pipe_escaped");
					// Replace with pipe
					add('|');
				}
				else if(is(pipe)){
					trace("pipe");
					// Enter `<script>` and push into `asciimath` state
					flush();
					auto span = node.append("span",{{"class","math"}});
					auto script = span.append("script",{{"type","math/asciimath"}});
					enter(script);
					push(asciimath);
				}
				else if(is(tex_open)){
					trace("tex_open");
					// Enter `<script>` and push into `tex` state
					flush();
					auto span = node.append("span",{{"class","math"}});
					auto script = span.append("script",{{"type","math/tex"}});
					enter(script);
					push(tex);
				}
				else if(is(link)){
					trace("link");
					// Flush text and append `<a>`
					flush();
					node.append("a").attr("href",match[4].str()).text(match[2].str());
				}
				else if(is(autolink)){
					trace("autolink");
					// Flush text and append `<a>`
					flush();
					node.append("a").attr("href",match.str()).text(match.str());
				}
				else if(is(endline)){
					trace("endline");
					// Move across into `sol` state
					across(sol);
				}
				else {
					trace("other");
					// Add character to buffer
					add();
				}
			}
			else if(state==empha){
				if(is(underscore)) exit_pop();
				else if(is(asterisk)) enter_push("strong",strong);
				else add();
			}
			else if(state==strong){
				if(is(asterisk)) exit_pop();
				else if(is(underscore)) enter_push("em",empha);
				else add();
			}
			else if(state==interp){
				if(is(backtick_backtick)){
					// Use buffer as `data-write` attribute, reset it,
					// then exit from `<span>` and pop up to `text` state
					node.attr("data-write",buffer);
					buffer = "";
					exit_pop();
				}
				else add();				
			}
			else if(state==code){
				if(is(backtick_escaped)) add('`');
				else if(is(backtick)) exit_pop();
				else add();
			}
			else if(state==asciimath){
				if(is(pipe_escaped)) add('|');
				else if(is(pipe)) exit_pop();
				else add();
			}
			else if(state==tex){
				if(is(tex_close)) exit_pop();
				else add();
			}
			else if(state==exec){
				// Capture all characters but on new lines
				// move to `sol` state to see if indentation
				// has reduced and should pop out of this state
				// @todo Remove leading indentation
				if(is(endline)){
					trace("endline");
					across(sol);
				}
				else add();
			}
			else add();
		}

		// Flush any remaining beffer to the current element
		flush();

		return *this;
	}
};

class CilaGenerator {
public:
	typedef Stencil::Node Node;

	/**
	 * Generate a string of Cila from a stencil
	 * 
	 * @param stencil Stencil to generate Cila for
	 */
	void generate(Node node, std::ostream& stream, const std::string& indent=""){
		if(node.is_document()){
			bool first = true;
			for(Node child : node.children()){
				if(not first) stream<<"\n";
				else first = false;
				generate(child,stream,indent);
			}
		}
		else if(node.is_element()){
			auto name = node.name();
			auto attrs = node.attrs();
			auto attrs_size = attrs.size();
			auto children = node.children();
			Node only_child;
			if(children.size()==1) only_child = children[0];

			// Shortcuts from whence we return...

			// Write directive
			if(name=="span" and children.size()==0 and attrs.size()==1 and node.attr("data-write").length()){
				stream<<"``"<<node.attr("data-write")<<"``";
				return;
			}
			// Emphasis & strong
			if((name=="em" or name=="strong") and attrs.size()==0){
				std::string delim;
				if(name=="em") delim = "_";
				else delim = "*";
				stream<<delim;
				for(Node child : node.children()) generate(child,stream);
				stream<<delim;
				return;
			}
			// Code
			if(name=="code" and attrs_size==0){
				auto text = node.text();
				boost::replace_all(text,"`","\\`");
				stream<<"`"<<text<<"`";
				return;
			}
			// Math
			if(name=="span" and node.attr("class")=="math"){
				auto script = node.select("script");
				auto code = script.text();
				auto type = script.attr("type");
				std::string begin,end;
				if(type=="math/asciimath"){
					begin = end = "|";
					boost::replace_all(code,"|","\\|");
				}
				if(type=="math/tex"){
					begin = "\\(";
					end = "\\)";
				}
				stream<<begin<<code<<end;
				return;
			}
			// Links and autolinks
			if(name=="a" and attrs_size==1 and node.has("href")){
				auto text = node.text();
				auto href = node.attr("href");
				if(text==href) stream<<text;
				else stream<<"["<<text<<"]("<<href<<")";
				return;
			}

			bool inlinee = Html::is_inline_element(name);

			// Keep track of whether content has been put to the stream for this
			// element for knowing if separating spaces are required
			bool separate = false;
			// Keep track of whether trailing text is allowed
			bool trail = true;

			// Paragraphs indicated by a preceding, indented, blank line
			if(name=="p" and children.size()>0 and attrs.size()==0){
				stream<<"\n"<<indent;
			}
			else {
				// Name
				auto tag = [&](){
					stream<<name;
					separate = true;
				};
				if(name=="span"){
					if(attrs.size()==1 and node.attr("data-write").length()){}
					else tag();
				}
				else if(name=="div"){
					if(attrs.size()==0 or node.attr("data-write").length()) tag();
				}
				else tag();
				// Attributes...
				for(auto name : attrs){
					auto value = node.attr(name);
					if(separate) stream<<" ";
					if(name=="id") stream<<"#"<<value;
					else if(name=="class") stream<<"."<<value;
					else if(
						name=="data-else" or 
						name=="data-default"
					) {
						stream<<name.substr(5);
						trail = false;
					}
					else if(
						name=="data-write" or 
						name=="data-with" or 
						name=="data-if" or 
						name=="data-elif" or 
						name=="data-switch" or 
						name=="data-case"
					){
						stream<<name.substr(5)<<" "<<value;
						trail = false;
					}
					else stream<<name<<"="<<value;
					separate = true;
				}
			}

			// Chillen
			if(trail and only_child and only_child.is_text()){			
				// Short text only child trails, long text only child is indented
				auto text = only_child.text();
				if(text.length()<100){
					if(separate) stream<<" ";
					stream<<text;
				}
				else stream<<"\n"<<indent<<"\t"<<text;
			} else {
				// Generate children
				for(Node child : node.children()){
					if(not inlinee) stream<<"\n"<<indent+"\t";
					generate(child,stream,indent+"\t");
				}
			}
		}
		else if(node.is_text()){
			std::string text = node.text();
			// Escape backticks and pipes
			boost::replace_all(text,"`","\\`");
			boost::replace_all(text,"|","\\|");
			stream<<text;
		}
		else {
			STENCILA_THROW(Exception,"Unhandled XML node type");
		}
	}

	std::string generate(Node node){
		std::stringstream cila;
		generate(node,cila);
		return cila.str();
	}

	std::string generate(const std::string& xml){
		Stencil stencil;
		stencil.xml(xml);
		return generate(stencil);
	}
};

}
