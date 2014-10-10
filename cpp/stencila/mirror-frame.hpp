#pragma once

#include <boost/format.hpp>

#include <stencila/mirror.hpp>
#include <stencila/traits.hpp>
#include <stencila/frame.hpp>

namespace Stencila {
namespace Mirrors {

class FrameReader : public Mirror<FrameReader> {
public:

	FrameReader(const Frame& frame, const std::vector<std::string>& exclude, const std::string& prefix=""):
		frame_(frame),
		prefix_(prefix),
		exclude_(exclude){}

	template<typename Data,typename... Args>
	FrameReader& data(Data& data, const std::string& name, Args... args){
		data_(data,name,IsStructure<Data>(),IsArray<Data>());
		return *this;
	}
  
private:
	template<typename Data>
	void data_(Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		if(read_(name)) FrameReader(frame_,exclude_,prefix_+name+".").mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		if(read_(name)) FrameReader(frame_,exclude_,prefix_+name).mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		if(read_(name)){
			std::string label = prefix_+name;
			if(frame_.has(label)) data = frame_.value<Data>(label);
		}
	}

	bool read_(const std::string& name){
		std::string label = prefix_+name;
		return std::find(exclude_.begin(), exclude_.end(), label)==exclude_.end();
	}

	const Frame& frame_;
	const std::vector<std::string>& exclude_;
	std::string prefix_;
}; // class FrameReader

}
}