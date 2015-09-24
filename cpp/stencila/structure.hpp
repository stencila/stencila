#pragma once

#include <boost/filesystem.hpp>

#include <stencila/polymorph.hpp>
#include <stencila/mirror-inspect.hpp>
#include <stencila/mirror-rows.hpp>
#include <stencila/mirror-formats.hpp>
#include <stencila/mirror-frame.hpp>

namespace Stencila {

template<class Derived>
class Structure : public Polymorph<Derived>{
public:

	using Polymorph<Derived>::derived;

	typedef bool structure_type;

	bool has(const std::string& name) const {
		return Mirrors::Has(name).mirror<Derived>();
	}

	std::vector<std::string> labels(void) const {
		return Mirrors::Labels().mirror<Derived>();
	}

	std::vector<std::string> values(void) const {
		return Mirrors::Labels().mirror<Derived>();
	}

	std::string json(void) {
		std::stringstream stream;
		Mirrors::JsonWriter().mirror(derived()).write(stream);
		return stream.str();
	}

	Derived& json(const std::string& json) {
		std::stringstream stream(json);
		Mirrors::JsonReader(stream).mirror(derived());
		return derived();
	}

	Derived& read(const std::string& path, std::string format = "", std::string directory = "") {
		auto pwd = boost::filesystem::current_path();
		if(directory.length()) boost::filesystem::current_path(directory);
		if(format.length()==0) format = boost::filesystem::extension(path).substr(1);
		if(format=="json") read_json(path);
		if(directory.length()) boost::filesystem::current_path(pwd);
		return derived();
	}

	Derived& write(const std::string& path, std::string format = "", std::string directory = "") {
		auto pwd = boost::filesystem::current_path();
		if(directory.length()){
			boost::filesystem::create_directories(directory);
			boost::filesystem::current_path(directory);
		}
		if(format.length()==0) format = boost::filesystem::extension(path).substr(1);
		if(format=="json") write_json(path);
		if(directory.length()) boost::filesystem::current_path(pwd);
		return derived();
	}

	Derived& read_json(const std::string& path) {
		std::ifstream stream(path);
		Mirrors::JsonReader(stream).mirror(derived());
		return derived();
	}

	Derived& write_json(const std::string& path) {
		std::ofstream stream(path);
		Mirrors::JsonWriter().mirror(derived()).write(stream);
		return derived();
	}

	Derived& read(const Frame& frame,const std::vector<std::string>& exclude = {}) {
		Mirrors::FrameReader(frame,exclude).mirror(derived());
		return derived();
	}

	Derived& write(Frame& frame,const std::vector<std::string>& exclude = {}) {
		Mirrors::FrameWriter(frame,exclude).mirror(derived());
		return derived();
	}

	std::string header_row(const std::string& separator="\t") const {
		return Mirrors::RowHeader(separator).mirror<Derived>();
	}

	std::string to_row(const std::string& separator="\t") {
		return Mirrors::RowGenerator(separator).mirror(derived());
	}

	Derived& from_row(const std::string& row, const std::string& separator="\t") {
		Mirrors::RowParser(row,separator).mirror(derived());
		return derived();
	}
};

} // namespace Stencila
