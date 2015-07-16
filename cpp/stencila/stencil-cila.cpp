#include <boost/regex.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/lexical_cast.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

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
		 * syntax (e.g. `[id="an-id"]`, `#an-id`, `.a-class`) including directives (e.g. `text x`) 
		 * and ignoring whitepace. If no attribute is found then moves across to `text` state.
		 */
		attrs,

		/**
		 * Looking for rendering flags (e.g. hash, index, off) som of which are only applied to
		 * directives:
		 *   - hash
		 *   - off
		 * and others which can be applied to both directives and normal elements
		 *   - index
		 *   - lock
		 *   - output
		 *   - included 
		 */
		flags,

		/**
		 * Text including shorthands and ilined elements
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
		 * Within an `exec` a `style` directive. Embedded code.
		 */
		embed
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
	Node stencil;

	/**
	 * Current indentation. Used for keeping track
	 * of parent-child relationships
	 */
	std::string indent;

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
	 * Bilge of characters which may me kept or discarded for
	 * embedded code elements
	 */
	std::string bilge;

	/**
	 * Flag for orphaned element attributes
	 */
	bool tag_needed;

	/**
	 * Flag for a paragraph is needed
	 */
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
			CASE(flags)
			CASE(text)
			CASE(empha)
			CASE(strong)
			CASE(interp)
			CASE(code)
			CASE(asciimath)
			CASE(tex)
			CASE(embed)
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
			#if defined(STENCILA_CILA_PARSER_TRACE)
				trace_show();
			#endif
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

		// Plain text at the start will get treated as a paragraph
		// (subsequently needs to have a blank line before it)
		para_needed = true;

		// Define regular expressions
		static const boost::regex
			indentation("[ \\t]*"),

			// Not necessary all tags, just those that are valid in stencils
			tag("("\
				"section|nav|article|aside|address|h1|h2|h3|h4|h5|h6|p|hr|pre|blockquote|ol|ul|li|dl|dt|dd|" \
				"figure|figcaption|div|a|em|strong|small|s|cite|q|dfn|abbr|data|time|code|var|samp|kbd|sub|sup|i|b|u|mark|ruby|" \
				"rt|rp|bdi|bdo|span|br|wbr|ins|del|table|caption|colgroup|col|tbody|thead|tfoot|tr|td|th|" \
				// Image elements
				"img|svg|" \
				// Form elements
				"form|fieldset|label|input|select|textarea|button"
			")\\b"),

			section(">\\s*([ \\w-]+)"),
			ul_item("-\\s*"),
			ol_item("\\d+\\.\\s*"),

			attr("\\[([\\w-]+)=(.+?)\\]"),
			id("#([\\w-]+)\\b"),
			clas("\\.([\\w-]+)\\b"),
			
			exec_open("(exec|js|r|py)\\b *([^:\\n]+)?(?=(: )|\\n|$)"),
			style_open("(style|css)(\\n|$)"),

			directive_noarg("(each|else|default)\\b *(?=(: )|\\n|\\{|\\}|$)"),
			directive_arg_optional("(comments)( +(.+?))?(?=(: )|\\n|\\{|\\}|$)"),
			directive_arg("(when|refer|attr|text|icon|with|if|elif|switch|case|for|include|delete|replace|change|before|after|prepend|append|macro|par|set|comment) +(.+?)(?=(: )|\\n|\\{|\\}|$)"),

			spaces(" +"),

			flags_open(": "),
			hash("&([a-zA-Z0-9]+)"),
			index("\\^(\\d+)"),
			error("\\!\\\"([^\\\"]*)\\\""),
			warning("\\%\\\"([^\\\"]*)\\\""),
			location("\\@(\\d+(,\\d+)?)"),
			lock("lock"),
			output("output"),
			off("off"),
			included("included"),

			empha_open("(\\s)_(?=[^\\s])"),
			empha_close("_"),
			strong_open("(\\s)\\*(?=[^\\s])"),
			strong_close("\\*"),

			tilde_escaped("\\\\~"),
			tilde("~"),

			backtick_escaped("\\\\`"),
			backtick("`"),

			pipe_escaped("\\\\\\|"),
			pipe("\\|"),

			tex_open("\\\\\\("),
			tex_close("\\\\\\)"),

			link("(\\[)([^\\]]*)(\\]\\()([^\\)]+)(\\))"),
			autolink("\\bhttp(s)?://[^ ]+\\b"),
			autoemail("[a-zA-Z0-9_-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9]+"),

			at_escaped("\\\\@"),
			refer("@([\\w-]+)\\b"),

			curly_open("\\{"),
			curly_close("\\}"),

			blankline("[ \\t]*\\n"),
			endline("\\n"),
			endinput("$")
		;

		trace_begin();
		while(begin!=end){
			trace_new();

			if(state==sol){
				// If this is not a blank line (zero or more spaces or tabs and nothing else)
				if(not boost::regex_search(begin, end, blankline, boost::regex_constants::match_continuous)){
					// Get indentation
					is(indentation);
					indent = match.str();
					// Peek ahead to see if this is a `li` shorthand line; for these
					// we don't want to pop off the parent `ul` or `ol`
					bool ul_li = boost::regex_search(begin, end, ul_item, boost::regex_constants::match_continuous);
					bool ol_li = boost::regex_search(begin, end, ol_item, boost::regex_constants::match_continuous);
					// Exit nodes until a node with lower indentation is reached
					// which then becomes the current node to which others get appended
					auto line_indent = indent.size();
					while(nodes.size()>1 and 
						(nodes.back().indent=="none" or line_indent<=nodes.back().indent.size())
					){
						auto node_indent = nodes.back().indent.size();
						if(ul_li and node.name()=="ul" and indent.size()==node_indent) break;
						if(ol_li and node.name()=="ol" and indent.size()==node_indent) break;
						exit();
					}
				}

				if(is(exec_open)){
					trace("exec");
					// An execute directive should only begin at the 
					// start of a line
					// Enter `<pre>` element and move across to `embed` state;
					enter_across("pre",embed);
					auto arg = match[1].str();
					if(match[2].str().length()) arg += " " + match[2].str();
					node.attr("data-exec",trim(arg));
				}
				else if(is(style_open)){
					trace("style");
					// A style directive should only begin at the 
					// start of a line
					// Enter `<style>` element and move across to `embed` state;
					enter_across("style",embed);
					std::string type = "text/css";
					node.attr("type",type);
					add("\n");
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
					if(node.name()!="ul") enter("ul");
					enter("li");
					across(text);
				}
				else if(is(ol_item)){
					trace("ol_item");
					// Enter `<ol>` if necessary, enter `<li>` and move into `text` state
					if(node.name()!="ol") enter("ol");
					enter("li");
					across(text);
				}
				else if(is(pipe)){
					trace("pipe");
					// Enter `<script>` and push into `asciimath` state
					flush();
					auto span = node.append("p",{{"class","equation"}});
					auto script = span.append("script",{{"type","math/asciimath; mode=display"}});
					enter(script);
					push(asciimath);
				}
				else if(is(tex_open)){
					trace("tex_open");
					// Enter `<script>` and push into `tex` state
					flush();
					auto span = node.append("p",{{"class","equation"}});
					auto script = span.append("script",{{"type","math/tex; mode=display"}});
					enter(script);
					push(tex);
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
					// Enter new element if necessary and create attribute;
					// keep on looking for more attributes
					enter_elem_if_needed();
					node.attr(match[1].str(),match[2].str());
				}
				else if(is(id)){
					trace("id");
					// Enter new element if necessary and create id attribute;
					// keep on looking for more attributes
					enter_elem_if_needed();
					node.attr("id",match[1].str());
				}
				else if(is(clas)){
					trace("clas");
					// Enter new element if necessary and create class attribute;
					// keep on looking for more attributes
					enter_elem_if_needed();
					node.concat("class",match[1].str());
				}
				else if(is(directive_noarg)){
					trace("directive_noarg");
					// Enter new element if necessary and create directive attribute;
					enter_elem_if_needed();
					node.attr("data-"+match[1].str(),"true");
				}
				else if(is(directive_arg_optional)){
					trace("directive_arg_optional");
					// Enter new element if necessary and create directive attribute;
					auto directive = match[1].str();
					auto arg = match[3].str();
					enter_elem_if_needed();
					node.attr("data-"+directive,trim(arg));
				}
				else if(is(directive_arg)){
					trace("directive_arg");
					// Enter new element if necessary and create directive attribute;
					// type of element depends on which directive;
					// move across to `flags` state (i.e no attributes or text to follow)
					auto directive = match[1].str();
					if(directive=="text" or directive=="refer") enter_elem_if_needed("span");
					else enter_elem_if_needed();
					std::string arg = match[2].str();
					boost::trim(arg);
					node.attr("data-"+directive,trim(arg));
				}
				else if(is(flags_open)){
					trace("flags");
					enter_elem_if_needed();
					across(flags);
				}
				else if(is(spaces)){
					trace("spaces");
					// Ignore spaces and keep on looking for attributes
				}
				else {
					trace("none");
					// If no match move across to `text`
					across(text);
				}
			}
			else if(state==flags){
				if(is(hash)){
					trace("hash");
					node.attr("data-hash",match[1].str());
				}
				else if(is(index)){
					trace("index");
					node.attr("data-index",match[1].str());
				}
				else if(is(error)){
					trace("error");
					node.attr("data-error",match[1].str());
				}
				else if(is(warning)){
					trace("warning");
					node.attr("data-warning",match[1].str());
				}
				else if(is(location)){
					trace("location");
					node.attr("data-location",match[1].str());
				}
				else if(is(lock)){
					trace("lock");
					node.attr("data-lock","true");
				}
				else if(is(output)){
					trace("output");
					node.attr("data-output","true");
				}
				else if(is(off)){
					trace("off");
					node.attr("data-off","true");
				}
				else if(is(included)){
					trace("included");
					node.attr("data-included","true");
				}
				else if(is(spaces)){
					trace("spaces");
					// Ignore spaces and keep on looking for flags
				}
				else {
					trace("none");
					// If current state is under an `embed` state then 
					// pop up to the `embed` otherwise move across to `sol`
					bool under_embed = false;
					if(states.size()>1){
						if(states[states.size()-2]==embed) under_embed = true;
					}
					if(under_embed){
						pop();
					} else {
						across(sol);
					}
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
				else if(is(empha_open)){
					trace("empha_open");
					// Add captured preceeding whitespace
					add(match[1].str());
					// Enter `<em>` and push into `empha` state
					enter_push("em",empha);
				}
				else if(is(strong_open)){
					trace("strong_open");
					// Add captured preceeding whitespace
					add(match[1].str());
					// Enter `<strong>` and push into `strong` state
					enter_push("strong",strong);
				}
				else if(is(tilde_escaped)){
					trace("tilde_escaped");
					// Replace with tilde
					add('~');
				}
				else if(is(tilde)){
					trace("tilde");
					// Enter a <span> and push into `interp` state
					enter_push("span",interp);
				}
				else if(is(backtick_escaped)){
					trace("backtick_escaped");
					// Replace with backtick
					add('`');
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
					auto script = node.append("script",{{"type","math/asciimath"}});
					enter(script);
					push(asciimath);
				}
				else if(is(tex_open)){
					trace("tex_open");
					// Enter `<script>` and push into `tex` state
					flush();
					auto script = node.append("script",{{"type","math/tex"}});
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
				else if(is(autoemail)){
					trace("autoemail");
					// Needs to be before `refer` to prevent @ begin matched there
					// Flush text and append a mailto link
					flush();
					node.append("a").attr("href","mailto:"+match.str()).text(match.str());
				}
				else if(is(at_escaped)){
					trace("at_escaped");
					// Replace with at
					add('@');
				}
				else if(is(refer)){
					trace("refer");
					// Flush text and append `<span data-refer="#id" />`
					flush();
					node.append("span").attr("data-refer","#"+match[1].str());
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
				if(is(empha_close)) exit_pop();
				else if(is(strong_open)){
					add(match[1].str());
					enter_push("strong",strong);
				}
				else add();
			}
			else if(state==strong){
				if(is(strong_close)) exit_pop();
				else if(is(empha_open)) {
					add(match[1].str());
					enter_push("em",empha);
				}
				else add();
			}
			else if(state==interp){
				if(is(tilde)){
					// Use buffer as `data-text` attribute, reset it,
					// then exit from `<span>` and pop up to `text` state
					node.attr("data-text",buffer);
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
			else if(state==embed){
				// Capture all characters but on new lines
				// move to `sol` state to see if indentation
				// has reduced and should pop out of this state
				// @todo Remove leading indentation
				if(is(flags_open)){
					trace("flags");
					push(flags);
				}
				else {
					static const boost::regex line("([ \t]*)([^\n]*)(\n|$)");
					boost::smatch match_local;
					boost::regex_search(begin, end, match_local, line, boost::regex_constants::match_continuous);
					auto indent_line = match_local[1].str();
					auto content_line = match_local[2].str();
					// Should this `embed` state end?
					if(content_line.length()>0 and indent_line.length()<=indent.length()){
						// Exit and pop. Note that `begin` is not shifted along at all
						// so that the line can be processed by `sol`
						exit();
						across(sol);
					} else {
						if(content_line.length()==0){
							// If this is an empty or blank (only whitespace chars) line then add a newline to the bilge
							// This means that whitespace chars on a blank line are considered insignificant; they are discarded
							bilge += "\n";
						} else {
							// Line is not empy, so use any bilge and add line to buffer
							// Add bilge to buffer and clear it
							buffer += bilge;
							bilge = "";
							// Add line to buffer
							if(indent_line.length()>=indent.length()+1) buffer += indent_line.substr(indent.length()+1);
							buffer += content_line;
							buffer += "\n";
						}
						// Shift along
						begin += match_local.position() + match_local.length();
					}
				}
			}
			else add();
		}

		// Flush any remaining beffer to the current element
		flush();

		return *this;
	}

	CilaParser& parse(Node node,const std::string& cila){
		stencil = node;
		node.clear();
		return parse(cila+"\n");
	}
};

class CilaGenerator {
public:
	typedef Stencil::Node Node;

	enum {
		NEWLINE,
		BLANKLINE,
		CONTENT
	} state;

	std::stringstream cila;

	/**
	 * Current indentation
	 */
	std::string indentation;

	/**
	 * Increase indentation
	 */
	void indent(void){
		indentation += "\t";
	}

	/**
	 * Decrease indentation
	 */
	void outdent(void){
		if(indentation.length()) indentation.pop_back();
	}

	/**
	 * Add line context
	 */
	void content(const std::string& content){
		if(state==NEWLINE){
			cila<<"\n"<<indentation;
		}
		else if(state==BLANKLINE){
			cila<<"\n\n"<<indentation;
		}
		cila<<content;
		state = CONTENT;
	}

	/**
	 * Create a new line (with the right indentation)
	 */
	void newline(void){
		if(state!=BLANKLINE) state = NEWLINE;
	}

	/**
	 * Create a blankline
	 */
	void blankline(void){
		state = BLANKLINE;
	}

	void visit(Node node, bool first=false, bool last=false){
		if(node.is_document()){
			visit_children(node);
		}
		else if(node.is_element()){
			auto name = node.name();
			auto children_list = node.children();
			auto children = children_list.size();
			auto attribute_list = node.attrs();
			auto attributes = attribute_list.size();

			// Remove an attribute already dealt with
			auto erase_attr = [&](const std::string& attr){
				attribute_list.erase(
					std::remove(attribute_list.begin(),attribute_list.end(),attr),
				attribute_list.end());
			};

			// Shortcuts from whence we return...and if we don't then the
			// default generation happens (that's why it's not an if,else if tree)
			
			// Write directive shorthand
			if(name=="span" and children==0 and attributes==1 and node.attr("data-text").length()){
				content("~"+node.attr("data-text")+"~");
				return;
			}
			// Refer directive shorthand
			if(name=="span" and children==0 and attributes==1 and node.attr("data-refer").length()){
				auto value = node.attr("data-refer");
				if(value[0]=='#'){
					int spaces = std::count_if(value.begin(), value.end(),[](unsigned char c){ return std::isspace(c); });
					if(spaces==0){
						content("@"+value.substr(1));
						return;
					}
				}
			}
			// Emphasis & strong
			if((name=="em" or name=="strong") and attributes==0){
				std::string delim;
				if(name=="em") delim = "_";
				else delim = "*";
				content(delim);
				visit_children(node,false);
				content(delim);
				return;
			}
			// Code
			if(name=="code" and attributes==0){
				auto text = node.text();
				boost::replace_all(text,"`","\\`");
				content("`"+text+"`");
				return;
			}
			// Equations and inline math
			if(name=="script" and node.attr("type")=="math/asciimath"){
				auto code = trim(node.text());
				boost::replace_all(code,"|","\\|");
				content('|'+code+'|');
				return;
			}
			if(name=="script" and node.attr("type")=="math/tex"){
				auto code = trim(node.text());
				content("\\("+code+"\\)");
				return;
			}
			// Links, autolinks and autoemails
			if(name=="a" and attributes==1 and node.has("href")){
				auto text = node.text();
				auto href = node.attr("href");
				if(text==href) content(text);
				else if(href.substr(0,7)=="mailto:" and href.substr(7)==text) content(text);
				else content("["+text+"]("+href+")");
				return;
			}
			// Lists with no attributes
			if((name=="ul" or name=="ol") and attributes==0 and children>0){
				// Only proceed if all children are `<li>`
				if(node.filter("li").size()==children){
					bool ol = name=="ol";
					int index = 0;
					for(auto child : children_list){
						if(index!=0) newline();
						index++;
						if(ol) content(index+". ");
						else content("- ");
						visit_children(child,false);
					}
					return;
				}
			}
			// Plain paragraph
			if(name=="p" and children>0 and attributes==0){
				blankline();
				visit_children(node,false);
				blankline();
				return;
			}
			// Equation paragraph
			if(name=="p" and node.attr("class")=="equation"){
				auto script = node.select("script");
				if(script){
					auto type = script.attr("type");
					if(type.length()){
						auto code = trim(script.text());
						std::string begin,end;
						if(type.find("math/asciimath")!=std::string::npos){
							begin = "|";
							end = "|";
							boost::replace_all(code,"|","\\|");
						} else {
							begin = "\\(";
							end = "\\)";
						}
						
						blankline();
						content(begin+code+end);
						blankline();
						return;
					}
				}
			}
			// Sections with an id attribute and a <h1> child
			if(name=="section" and node.attr("id").length() and children>0){
				// Only proceed if <h1> is first child
				if(children_list[0].name()=="h1"){
					// Only proceed if id is consistent with header
					auto h1 = node.select("h1");
					auto title = h1.text();
					auto id_expected = title;
					boost::trim(id_expected);
					boost::to_lower(id_expected);
					boost::replace_all(id_expected," ","-");
					auto id = node.attr("id");
					if(id==id_expected){
						// Add shorthand with blank line before
						blankline();
						content("> "+boost::trim_copy(title));
						// Generate each child on a new line except for the h1
						indent();
						unsigned int index = 0;
						unsigned int last = children-1;
						for(Node child : node.children()){
							if(not(child.name()=="h1" and child.text()==title)){
								newline();
								visit(child,index==0,index==last);
								index++;
							}
						}
						outdent();
						return;
					}
				}
			}
				
			// Everything that could not be shorthandted still remains here...
		
			bool separate = false;
			bool trail = true;
			bool embedded = false;
			bool blank_after = false;

			// Start of line depends on type of element...
			// Execute directives
			if(node.has("data-exec")){
				blankline();
				content(node.attr("data-exec"));
				separate = true;

				erase_attr("data-exec");
				embedded = true;
				blank_after = true;
			}
			// Style elements
			else if(name=="style"){
				std::string lang = "css";
				std::string type = node.attr("type");
				if(type=="text/css") lang = "css";
				
				blankline();
				content(lang);
				separate = true;

				erase_attr("type");
				embedded = true;
				blank_after = true;
			}
			// <div>s only need to be specified if 
			// 	- no attributes following
			// 	- not a `text` or `refer` directive (which have span defaults)
			else if(name=="div"){
				if(attributes==0 or node.has("data-text") or node.has("data-refer")){
					content(name);
					separate = true;
				}
			}
			// <span>s don't need to specified if a `text` or `refer` directive
			else if(name=="span"){
				if(not (node.has("data-text") or node.has("data-refer"))){
					content(name);
					separate = true;
				}
			}
			else if(
				node.has("data-when") or node.has("data-with") or node.has("data-for")
				or node.has("data-switch") or node.has("data-include") or node.has("data-macro")
			){
				blankline();
				content(name);
				separate = true;
				blank_after = true;				
			}
			else {
				content(name);
				separate = true;
			}

			// Handle attributes...
			if(attributes){
				std::pair<std::string,std::string> directive;
				std::vector<std::pair<std::string,std::string>> flags;
				for(auto name : attribute_list){
					auto value = node.attr(name);
					if(Stencil::directive(name)){
						directive.first = name;
						directive.second = value;
					}
					else if(Stencil::flag(name)){
						flags.push_back({name,value});
					}
					else {
						if(separate) content(" ");

						if(name=="id"){
							content("#"+value);
						}
						else if(name=="class"){
							// Get class attribute and split using spaces
							std::vector<std::string> classes;
							boost::split(classes,value,boost::is_any_of(" "));
							int index = 0;
							for(auto name : classes){
								if(index>0) content(" ");
								if(name.length()) content("."+name);
								index++;
							}
						}
						else {
							content("["+name+"="+value+"]");
						}

						separate = true;
					}
				}

				// Directives
				if(directive.first.length()){
					if(separate) content(" ");
					auto name = directive.first;
					content(name.substr(5));
					if(not(name=="data-each" or name=="data-else" or name=="data-default")){
						auto value = directive.second;
						content(" "+value);
					}
					trail = false;
					separate = true;
				}

				// Flags
				if(flags.size()){
					if(separate) content(" ");
					content(":");
				}
				for(auto attr : flags){
					auto name = attr.first;
					auto value = attr.second;
					std::string flag;
					if(name=="data-hash") flag = "&"+value;
					else if(name=="data-index") flag = "^"+value;
					else if(name=="data-error") flag = "!\""+replace_all(value,"\"","'")+"\"";  // Double quote replaced with single to avoid parsing errors
					else if(name=="data-warning") flag = "%\""+replace_all(value,"\"","'")+"\"";
					else if(name=="data-location") flag = "@"+value;
					else flag = name.substr(5);
					content(" "+flag);
					trail = false;
					separate = true;
				}
			}

			if(not embedded){
				// Short text only child trails, long text only child is indented
				if(trail and children==1){
					auto child = children_list[0];
					if(child.is_text()){			
						auto text = child.text();
						boost::trim(text);
						if(text.length()<100){
							if(separate) content(" ");
							content(text);
						}
						else {
							indent();
							newline();
							content(text);
							outdent();
						}
						return;
					}
				}
				// Otherwise all childen indented
				indent();
				visit_children(node);
				outdent();
			} else {
				// Get the code from the child nodes. Usually there will be only one, but in case there are more
				// add them all. Note that the text() method unencodes HTML special characters (e.g. &lt;) for us
				std::string code;
				for(Node child : node.children()) code += child.text();
				// Trim white space (it should never be significant when at start or end)
				// Normally code will start and end with a new line (that is how it is created when parsed)
				// so remove those, and any other whitespace, for consistent Cila generation
				boost::trim(code);
				if(code.length()>0){
					// Split into lines
					std::vector<std::string> lines;
					boost::split(lines,code,boost::is_any_of("\n"));
					// Output each line, with extra indentation if it has content
					indent();
					for(unsigned int index=0;index<lines.size();index++){
						auto line = lines[index];
						if(line.length()>0){
							newline();
							content(line);
						} else {
							content("\n");
						}
					}
					outdent();
				}
			}

			// Add following blank line
			if(blank_after) blankline();
		}
		else if(node.is_text()){
			auto text = node.text();
			// Trim white space if first or last child
			if(first) boost::trim_left(text);
			if(last) boost::trim_right(text);
			// Escape characters used for shorthands
			boost::replace_all(text,"`","\\`");
			boost::replace_all(text,"|","\\|");
			boost::replace_all(text,"~","\\~");
			boost::replace_all(text,"@","\\@");

			content(text);
		}
		else {
			STENCILA_THROW(Exception,"Unhandled XML node type");
		}
	}

	void visit_children(Node node, bool newlines=true){
		int index = 0;
		int last = node.children().size()-1;
		for(Node child : node.children()){
			if(newlines) newline();
			visit(child,index==0,index==last);
			index++;
		}
	}

	std::string generate(Node node){
		cila.str("");
		visit(node);
		return trim(cila.str());
	}

};

#if !defined(STENCILA_CILA_INLINE)

Stencil& Stencil::cila(const std::string& string){
	CilaParser().parse(*this,string);
	return *this;
}

std::string Stencil::cila(void) const {
	return CilaGenerator().generate(*this);
}

#endif

}
