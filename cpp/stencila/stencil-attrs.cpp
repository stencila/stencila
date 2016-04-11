#include <algorithm>

#include <boost/regex.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

std::string Stencil::title(void) const {
	return select("#title").text();
}

std::string Stencil::description(void) const {
	return select("#description").text();
}

std::vector<std::string> Stencil::keywords(void) const {
	std::vector<std::string> keywords;
	if(Node elem = select("#keywords")){
		auto text = elem.text();
		keywords = split(text,",");
		for(auto& keyword : keywords) trim(keyword);
	}
	return keywords;
}

std::vector<std::string> Stencil::authors(void) const {
	std::vector<std::string> authors;
	for(auto& author : filter(".author")){
		authors.push_back(author.text());
	}
	return authors;
}

std::string Stencil::mode(void) const {
	return select("#mode").text();
}

std::string Stencil::environ(void) const {
	auto list = environs();
	if (list.size()) return list.front();
	else return "";
}

std::vector<std::string> Stencil::environs(void) const {
	std::vector<std::string> environs;
	if(Node elem = select("#environs")){
		// A #environs node found so use that
		auto text = elem.text();
		environs = split(text,",");
		for(auto& environ : environs) trim(environ);
	} else {
		// Count the number of exec directives of each type
		std::map<std::string,int> counts;
		for(auto exec : execs()){
			for(auto environ : exec.contexts){
				if(counts.find(environ)==counts.end()) counts[environ] = 1;
				else counts[environ] += 1;
			}
		}
		// Sort in decending order of count
		std::vector<std::pair<std::string,int>> sorted;
		for(auto pair : counts) sorted.push_back(pair);
		auto cmp = [](std::pair<std::string,int> const & a, std::pair<std::string,int> const & b){ 
			return a.second > b.second;
		};
		std::sort(sorted.begin(), sorted.end(), cmp);
		for(auto pair : sorted) environs.push_back(pair.first);

	}   
	return environs;
}

std::string Stencil::theme(bool versioned) const {
	if(auto theme = select("#theme")){
		auto value = theme.text();
		if(versioned) return value;
		else {
			auto parts = split(value,"==");
			return parts[0];
		}
	}
	else return "core/stencils/themes/default";
}

}
