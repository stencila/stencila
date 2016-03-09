#include <iostream>

#include <stencila/component.hpp>
#include <stencila/hub.hpp>
#include <stencila/helpers.hpp>

namespace Stencila {

Component& Component::store(void) {
	// FIXME: a temporary implementation using shell commands
	using Helpers::execute;
	auto tar = Host::temp_filename("tgz");
	auto command = std::string("cd ") + path(true) +
		" && tar -czf " + tar + " *" +
		" && curl -s -X POST -H 'Accept:application/json'" +
			" -u Token:" + hub.token() +
			" -F 'file=@" + tar + "'" + 
			" " + hub.origin() + "/" + address(true) + "@snapshot";

	std::cout<<"Storing "<<std::flush;
	execute(command);
	std::cout<<std::endl;

	return *this;
}

Component& Component::restore(void) {
	// FIXME: a temporary implementation using shell commands
	using Helpers::execute;
	auto command = std::string("cd ") + path(true) +
		" && curl -s -L -H 'Accept:application/json'" +
			" -u Token:" + hub.token() +
			" " + hub.origin() + "/" + address(true) + "@snapshot" +
		" | tar -xz";

	std::cout<<"Restoring "<<std::flush;
	execute(command);
	std::cout<<std::endl;

	return *this;
}

}
