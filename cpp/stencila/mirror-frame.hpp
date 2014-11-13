#pragma once

#include <stencila/frame.hpp>
#include <stencila/mirror.hpp>
#include <stencila/traits.hpp>

namespace Stencila {
namespace Mirrors {

template<class Derived>
class FrameIoer : public Mirror<Derived> {
public:

	FrameIoer(const Frame& frame, const std::vector<std::string>& exclude, const std::string& prefix=""):
		frame_(frame),
		prefix_(prefix),
		exclude_(exclude){}

	using Mirror<Derived>::derived;

	template<typename Data,typename... Args>
	FrameIoer& data(Data& data, const std::string& name, Args... args){
		derived().data_(data,name,IsStructure<Data>(),IsArray<Data>());
		return *this;
	}

protected:

	bool handle_(const std::string& name){
		std::string label = prefix_+name;
		return std::find(exclude_.begin(), exclude_.end(), label)==exclude_.end();
	}

	const Frame& frame_;
	std::string prefix_;
	const std::vector<std::string>& exclude_;
}; // class FrameIoer


class FrameReader : public FrameIoer<FrameReader> {
public:
	FrameReader(const Frame& frame, const std::vector<std::string>& exclude, const std::string& prefix=""):
		FrameIoer(frame,exclude,prefix){}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		if(handle_(name)) FrameReader(frame_,exclude_,prefix_+name+".").mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		if(handle_(name)) FrameReader(frame_,exclude_,prefix_+name).mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		if(handle_(name)){
			std::string label = prefix_+name;
			if(frame_.has(label)) data = frame_(0,label);
		}
	}
}; // class FrameReader

class FrameWriter : public FrameIoer<FrameWriter> {
public:
	FrameWriter(const Frame& frame, const std::vector<std::string>& exclude, const std::string& prefix=""):
		FrameIoer(frame,exclude,prefix){}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		if(handle_(name)) FrameWriter(frame_,exclude_,prefix_+name+".").mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		if(handle_(name)) FrameWriter(frame_,exclude_,prefix_+name).mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		if(handle_(name)){
			frame_.add(prefix_+name,data);
		}
	}
}; // class FrameWriter

}
}