#include <stencila/theme.hpp>

namespace Stencila {

std::string Theme::title(void) const {
	return title_;
}

std::string Theme::description(void) const {
	return description_;
}

std::vector<std::string> Theme::keywords(void) const {
	return keywords_;
}

std::vector<std::string> Theme::authors(void) const {
	return authors_;
}

std::string Theme::theme(void) const {
	return theme_;
}

}
