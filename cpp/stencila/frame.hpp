#pragma once

#include <string>
#include <vector>
#include <fstream>

#include <boost/any.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/format.hpp>

#include <stencila/exception.hpp>
#include <stencila/datatype.hpp>

namespace Stencila {

class Frame {
public:

	typedef boost::any Type;

	Frame(void){
	}

	Frame(const std::vector<std::string>& labels, unsigned int rows=0){
		initialise(rows,labels);
	}

	Frame(unsigned int rows, const std::vector<std::string>& labels={}){
		initialise(rows,labels);
	}

	template<typename Type>
	static Frame of(void){
		auto labels  = static_cast<Type*>(nullptr)->labels();
		Frame frame;
		for(auto label : labels) frame.add(label,Null);
		return frame;
	}

	Frame& initialise(unsigned int rows, const std::vector<std::string>& labels){
		rows_ = rows;
		labels_ = labels;
		columns_ = labels_.size();
		resize();
		return *this;
	}

	Frame& resize(void){
		data_.resize(rows_,std::vector<boost::any>(columns_));
		return *this;
	}

	unsigned int rows(void) const {
		return rows_;
	}

	unsigned int columns(void) const {
		return columns_;
	}

	std::vector<std::string> labels(void) const {
		return labels_;
	}

	std::string label(unsigned int index) const {
		return labels_[index];
	}

	int column(const std::string& label) const {
		auto iter = std::find(labels_.begin(),labels_.end(),label);
		if(iter==labels_.end()) return -1;
		else return iter-labels_.begin();
	}

	bool has(const std::string& label) const {
		return column(label)>=0;
	}

	std::vector<Datatype> types(void) const {
		return types_;
	}

	Datatype type(unsigned int column) const {
		return types_[column];
	}

	Datatype type(unsigned int row, unsigned int column) const {
		return Datatype::from_type_info(data_[row][column].type());
	}

	template<typename Type>
	Frame& type(void){
		for(unsigned int row=0;row<rows_;row++){
			for(unsigned int col=0;col<columns_;col++){
				data_[row][col] = boost::lexical_cast<Type>(boost::any_cast<std::string>(data_[row][col]));
			}
		}
		return *this;
	}	

	template<typename Type=boost::any>
	Frame& add(const std::string& label, const Datatype& type, const Type& value=Type()){
		columns_++;
		labels_.push_back(label);
		types_.push_back(type);
		for(unsigned int row=0;row<rows_;row++) data_[row].push_back(value);
		return *this;
	}

	Frame& append(void){
		data_.push_back(std::vector<boost::any>(columns_));
		rows_++;
		return *this;
	}

	Frame& append(const std::vector<boost::any>& row){
		if(row.size()!=columns_){
			STENCILA_THROW(Exception,str(boost::format(
				"Error attempting to append a row with <%i> columns to a frame with <%s> columns"
			)%row.size()%columns_));
		}
		data_.push_back(row);
		rows_++;
		return *this;
	}

	template<typename Type>
	Frame& append(const std::vector<Type>& row){
		if(row.size()!=columns_){
			STENCILA_THROW(Exception,str(boost::format(
				"Error attempting to append a row with <%i> columns to a frame with <%s> columns"
			)%row.size()%columns_));
		}
		std::vector<boost::any> values(columns_);
		for(int col=0;col<columns_;col++) values[col] = row[col];
		data_.push_back(values);
		rows_++;
		return *this;
	}

	Frame& append(const Frame& frame){
		if(columns_==0){
			initialise(0,frame.labels_);
		}
		else if(frame.columns()!=columns_){
			STENCILA_THROW(Exception,str(boost::format(
				"Error attempting to append a frame with <%i> columns to a frame with <%s> columns"
			)%frame.columns()%columns_));
		}
		for(unsigned int row=0;row<frame.rows();row++){
			std::vector<boost::any> data(columns_);
			for(unsigned int col=0;col<frame.columns();col++) data[col] = frame(row,col);
			data_.push_back(data);
			rows_++;
		}
		return *this;
	}

	boost::any& operator()(unsigned int row, unsigned int column){
		return data_[row][column];
	}

	const boost::any& operator()(unsigned int row, unsigned int column) const {
		return data_[row][column];
	}

	Frame row(unsigned int row) const {
		Frame frame(1,labels());
		frame.data_[0] = data_[row];
		return frame;
	}

	template<typename Type>
	Type value(unsigned int row, unsigned int column) const {
		return boost::any_cast<Type>(data_[row][column]);
	}

	template<typename Type>
	Type value(const std::string& label) const {
		int col = column(label);
		if(col<0) STENCILA_THROW(Exception,"No such label <"+label+"> in frame")
		return boost::any_cast<Type>(data_[0][col]);
	}

	static boost::any from_string(const std::string& string, Datatype datatype) {
		boost::any any;
		switch(datatype.code){
			case 'n': break;
			case 'i': any = boost::lexical_cast<int>(string); break;
			case 'r': any = boost::lexical_cast<double>(string); break;
			case 't': any = string; break;
		}
		return any;
	}

	std::string string(unsigned int row, unsigned int column) const {
		auto any = data_[row][column];
		auto datatype = type(row,column);
		switch(datatype.code){
			case 'n': return "";
			case 'i': return boost::lexical_cast<std::string>(boost::any_cast<int>(any));
			case 'r': return boost::lexical_cast<std::string>(boost::any_cast<double>(any));
			case 't': return boost::any_cast<std::string>(any);
		}
		return "";
	}

	void clear(const std::string path) {
		data_.clear();
	}

	Frame& read(std::istream& stream, const std::string& separator="\t") {
		// Get labels from header and use to intialise
		std::string header;
		std::getline(stream,header);
		std::vector<std::string> labels;
		boost::split(labels,header,boost::is_any_of(separator));
		initialise(0,labels);
		// Get each line....
		std::string line;
		while(std::getline(stream,line)){
			// Skip lines that are all whitespace
			// (this primarily is to prevent errors caused by extra empty lines at end of files)
			if(std::all_of(line.begin(),line.end(),isspace)) continue;
			// Split into values
			std::vector<std::string> values;
			boost::split(values,line,boost::is_any_of(separator));
			append(values);
		}
		return *this;
	}

	Frame& read(const std::string path, const std::string& separator="\t") {
		std::ifstream file(path);
		return read(file,separator);
	}

	const Frame& write(const std::string path) const {
		std::ofstream file(path);
		for(unsigned int col=0;col<columns_;col++){
			file<<labels_[col];
			if(col!=columns_-1) file<<"\t";
		}
		file<<"\n";
		for(unsigned int row=0;row<rows_;row++){
			for(unsigned int col=0;col<columns_;col++){
				file<<string(row,col);
				if(col!=columns_-1) file<<"\t";
			}
			file<<"\n";
		}
		return *this;
	}

private:
	std::vector<std::vector<boost::any>> data_;
	unsigned int rows_ = 0;
	unsigned int columns_ = 0;
	std::vector<std::string> labels_;
	std::vector<Datatype> types_;
};

}