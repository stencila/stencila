#pragma once

#include <string>
#include <vector>
#include <fstream>

#include <boost/any.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/format.hpp>

#include <stencila/exception.hpp>
#include <stencila/frame-declaration.hpp>
#include <stencila/datatype.hpp>

namespace Stencila {

template<>
class Frame<> {
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
	static Frame<> of(void){
		auto labels  = static_cast<Type*>(nullptr)->labels();
		Frame<> frame;
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

	std::vector<Datatype> types(void) const {
		return types_;
	}

	Datatype type(unsigned int column) const {
		return types_[column];
	}

	Datatype type(const boost::any& any) const {
		const std::type_info& type = any.type();
		if(type==typeid(void)) return Null;
		if(type==typeid(int)) return Integer;
		if(type==typeid(float)) return Real;
		if(type==typeid(double)) return Real;
		if(type==typeid(std::string)) return Text;
		STENCILA_THROW(Exception,str(boost::format("Unrecognised type <%s>")%type.name()));
	}

	Datatype type(unsigned int row, unsigned int column) const {
		return type(data_[row][column]);
	}

	Frame& add(const std::string& label, const Datatype& type){
		columns_++;
		labels_.push_back(label);
		types_.push_back(type);
		for(unsigned int row=0;row<rows_;row++) data_[row].push_back(boost::any());
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
		if(frame.columns()!=columns_){
			STENCILA_THROW(Exception,str(boost::format(
				"Error attempting to append a frame with <%i> columns to a frame with <%s> columns"
			)%frame.columns()%columns_));
		}
		for(unsigned int row=0;row<frame.rows();row++){
			std::vector<boost::any> data(columns_);
			for(unsigned int col=0;col<frame.columns();col++) data[col] = frame(row,col);
			data_.push_back(data);
		}
		return *this;
	}

	boost::any& operator()(unsigned int row, unsigned int column){
		return data_[row][column];
	}

	const boost::any& operator()(unsigned int row, unsigned int column) const {
		return data_[row][column];
	}

	template<typename Type>
	Type value(unsigned int row, unsigned int column) const {
		return boost::any_cast<Type>(data_[row][column]);
	}

	std::string string(unsigned int row, unsigned int column) const {
		auto any = data_[row][column];
		auto datatype = type(any);
		switch(datatype.code){
            case 'n': return "";
            case 'i': return boost::lexical_cast<std::string>(value<int>(row,column));
            case 'r': return boost::lexical_cast<std::string>(value<double>(row,column));
            case 't': return value<std::string>(row,column);
        }
		return "";
	}

	void write(const std::string path) const {
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
	}

private:
	typedef std::vector<boost::any> Column;
	std::vector<Column> data_;
	unsigned int rows_ = 0;
	unsigned int columns_ = 0;
	std::vector<std::string> labels_;
	std::vector<Datatype> types_;
};

}