#include <stencila/component.hpp>
#include <stencila/hub.hpp>
#include <stencila/helpers.hpp>

#include <iostream>

namespace Stencila {

Component& Component::store(void) {
	// FIXME: a temporary implementation using shell commands
	using Helpers::execute;
	auto tar = Host::temp_filename("tar.gz");
	auto command = std::string("cd ") + Component::path(true) +
		" && tar -czf " + tar + " *"
		" && curl -s -X POST -H 'Accept:application/json'" +
			" -u Token:" + Host::env_var("STENCILA_HUB_TOKEN") +
			" -F 'file=@" + tar + "'" + 
			" " + Host::env_var("STENCILA_HUB_ROOT") + "/" + Component::address(true) + "@snapshot > /dev/null";
	execute(command);
	return *this;
}

Component& Component::restore(void) {
	// FIXME: a temporary implementation using shell commands
	using Helpers::execute;
	auto command = std::string("cd ") + Component::path(true) +
		" && curl -s -L -H 'Accept:application/json'" +
			" -u Token:" + Host::env_var("STENCILA_HUB_TOKEN") +
			" " + Host::env_var("STENCILA_HUB_ROOT") + "/" + Component::address(true) + "@snapshot" +
		" | tar -xz";
	try {
		execute(command);
	} catch (Stencila::Exception e) {
		// Clumsily deal with failiure of command due to no exisiting snapshot
		// for component
	}
	return *this;
}

}

