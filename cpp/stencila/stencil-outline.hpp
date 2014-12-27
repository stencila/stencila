#include <stencila/stencil.hpp>

namespace Stencila {

struct Stencil::Outline {

	struct Level {

		Level* parent = nullptr;
		int level = 0;
		int index;
		std::string label;
		std::string id;
		std::vector<Level*> sublevels;

		~Level(void){
			for(auto* level : sublevels) delete level;
		}

		Level* sublevel(void){
			Level* sublevel = new Level;
			sublevel->level = level+1;
			sublevel->index = sublevels.size()+1;
			sublevel->parent = this;
			sublevels.push_back(sublevel);
			return sublevel;
		}

		std::string path(const std::string& sep=".") const {
			std::string path;
			const Level* next = this;
			while(next->index){
				if(path.length()) path.insert(0,sep);
				path.insert(0,string(next->index));
				next = next->parent;
			}
			return path;
		}

		std::string id_(void) const {
			return "section-"+path("-");
		}

		std::string class_(void) const {
			return "level-"+string(level);
		}

		void heading(Node node){
			if(label.length()==0){
				// Get label for this level from the node
				label = node.text();
				// Check for node id, create one if needed, then add it to 
				// level for links and to the section header
				auto id_value = node.attr("id");
				if(not id_value.length()){
					id_value = id_();
					node.attr("id",id_value);
				}
				id = id_value;
				// Check for an existing label
				std::string path_string = path();
				Node label = node.select(".label");
				if(not label){
					// Prepend a label
					label = node.prepend("span");
					label.attr("class","label");
					label.append("span",{{"class","path"}},path_string);
					label.append("span",{{"class","separator"}}," ");
				} else {
					// Ammend the label
					Node path = label.select(".path");
					if(not path) path = label.append("span",{{"class","path"}},path_string);
					else path.text(path_string);
				}            
				// Give class to the heading for styling
				node.attr("class",class_());
			}
		}

		void render(Node ul) const {  
			Node li = ul.append(
				"li",
				{{"class",class_()}}
			);
			li.append(
				"a",
				{{"href","#"+id}},
				path()+" "+label
			);
			for(auto* level : sublevels) level->render(ul);
		}
	};

	Level* root;
	Level* current;
	Node node;

	Outline(void){
		root = new Level;
		current = root;
	}

	~Outline(void){
		if(root) delete root;
	}

	void enter(void){
		current = current->sublevel();
	}

	void exit(void){
		current = current->parent;
	}

	void heading(Node node){
		current->heading(node);
	}

	void render(void){
		if(node) {
			Node ul = node.append("ul");
			root->render(ul);
		}
	}
};

}