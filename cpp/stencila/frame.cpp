#define STENCILA_FRAME_CPP

#include <fstream>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
//#define BOOST_DISABLE_ASSERTS
#include <boost/multi_array.hpp>

#include <stencila/exception.hpp>
#include <stencila/frame.hpp>
#include <stencila/string.hpp>

namespace Stencila {

Frame::Frame(void):
	data_(new Data){
	resize_(0,0);
}

Frame::Frame(const Frame& frame):
	data_(new Data(*frame.data_)),
	labels_(frame.labels_){
	resize_(frame.rows(),frame.columns());
}

Frame::Frame(const std::vector<std::string>& labels, unsigned int rows):
	data_(new Data),
	labels_(labels){
	resize_(rows,labels_.size());
}

Frame::Frame(unsigned int rows, const std::vector<std::string>& labels):
	data_(new Data),
	labels_(labels){
	resize_(rows,labels_.size());
}

Frame::Frame(const std::vector<std::string>& labels, const std::vector<double>& values):
	data_(new Data),
	labels_(labels){
	unsigned int cols = labels.size();
	unsigned int rows = values.size()/labels.size();
	resize_(rows,cols);
	for(unsigned int row=0;row<rows;row++){
		for(unsigned int col=0;col<cols;col++){
			operator()(row,col) = values[row*cols+col];
		}
	}
}

Frame::~Frame(void){
	delete data_;
};

unsigned int Frame::rows(void) const {
	return data_->shape()[0];
}

unsigned int Frame::columns(void) const {
	return data_->shape()[1];
}

bool Frame::empty(void) const {
	return rows()==0;
}

std::vector<std::string> Frame::labels(void) const {
	return labels_;
}

std::string Frame::label(unsigned int index) const {
	return labels_[index];
}

int Frame::label(const std::string& label) const {
	auto iter = std::find(labels_.begin(),labels_.end(),label);
	if(iter==labels_.end()) return -1;
	else return iter-labels_.begin();
}

bool Frame::has(const std::string& label) const {
	return Frame::label(label)>=0;
}

double& Frame::operator()(unsigned int row, unsigned int column){
	return (*data_)(boost::array<Data::index,2>{{row,column}});
}

const double& Frame::operator()(unsigned int row, unsigned int column) const {
	return (*data_)(boost::array<Data::index,2>{{row,column}});
}

double& Frame::operator()(unsigned int row, const std::string& label) {
	return operator()(row,this->label(label));
}

const double& Frame::operator()(unsigned int row, const std::string& label) const {
	return operator()(row,this->label(label));
}

std::vector<double> Frame::row(unsigned int row) const {
	return std::vector<double>();
}

std::vector<double> Frame::column(unsigned int column) const {
	return std::vector<double>();
}

std::vector<double> Frame::column(const std::string& label) const {
	return column(Frame::label(label));
}

Frame Frame::slice(unsigned int row) const {
	Frame frame(1,labels());
	for(unsigned int col=0;col<columns();col++) frame(0,col) = operator()(row,col);
	return frame;
}

Frame& Frame::add(const std::string& label, const double& value){
	labels_.push_back(label);
	delta_(0,1);
	unsigned int column = columns() - 1;
	for(unsigned int row=0;row<rows();row++) (*data_)[row][column] = value;
	return *this;
}

Frame& Frame::append(unsigned int rows){
	delta_(rows,0);
	return *this;
}

Frame& Frame::append(const std::vector<double>& values){
	auto cols = columns();
	if(values.size()!=cols){
		STENCILA_THROW(Exception,str(boost::format(
			"Error attempting to append a row with <%i> columns to a frame with <%i> columns"
		)%values.size()%cols));
	}
	delta_(1,0);
	auto row = rows()-1;
	for(unsigned int col=0;col<cols;col++) operator()(row,col) = values[col];
	return *this;
}

Frame& Frame::append(const std::vector<std::string>& values){
	std::vector<double> numbers(values.size());
	for(unsigned int i=0;i<values.size();i++){
		auto string = values[i];
		double number;
		try {
			number = boost::lexical_cast<double>(string);
		} catch(...){
			STENCILA_THROW(Exception,"Error attempting to convert string <"+string+"> to number");
		}
		numbers[i] = number;
	}
	return append(numbers);
}

Frame& Frame::append(const Frame& frame){
	if(columns()==0){
		labels_ = frame.labels();
		resize_(0,labels_.size());
	}
	else if(frame.columns()!=columns()){
		STENCILA_THROW(Exception,str(boost::format(
			"Error attempting to append a frame with <%i> columns to a frame with <%s> columns"
		)%frame.columns()%columns()));
	}
	unsigned int rows_old = rows();
	delta_(frame.rows(),0);
	unsigned int rows_new = rows();
	(*data_)[boost::indices
		[Data::index_range(rows_old,rows_new)]
		[Data::index_range(0,columns())]
	] = *frame.data_;
	return *this;
}

Frame& Frame::clear(void){
	resize_(0,0);
	return *this;
}

Frame& Frame::read(std::istream& stream, const std::string& separator) {
	// Clear this frame
	clear();
	// Get labels from header and use to intialise
	std::string header;
	std::getline(stream,header); 
	std::vector<std::string> labels;
	boost::split(labels,header,boost::is_any_of(separator));
	labels_ = labels;
	resize_(0,labels_.size());
	// Get each line 
	std::string line;
	int count = 0;
	while(std::getline(stream,line)){
		try {
			count++;
			// Skip lines that are all whitespace
			// (this primarily is to prevent errors caused by extra empty lines at end of files)
			if(std::all_of(line.begin(),line.end(),isspace)) continue;
			// Split into values
			std::vector<std::string> values;
			boost::split(values,line,boost::is_any_of(separator));
			append(values);
		} catch (const std::exception& error){
			STENCILA_THROW(Exception,"Error reading line.\n  number: "+string(count)+"\n  content: "+line.substr(0,20)+"...\n  error: "+error.what());
		}
	}
	return *this;
}

Frame& Frame::read(const std::string path, const std::string& separator) {
	std::ifstream file(path);
	return read(file,separator);
}

const Frame& Frame::write(std::ostream& stream,const std::string& separator) const {
	auto rows = Frame::rows();
	auto cols = Frame::columns();
	for(unsigned int col=0;col<cols;col++){
		stream<<label(col);
		if(col!=cols-1) stream<<"\t";
	}
	stream<<"\n";
	for(unsigned int row=0;row<rows;row++){
		for(unsigned int col=0;col<cols;col++){
			stream<<operator()(row,col);
			if(col!=cols-1) stream<<"\t";
		}
		stream<<"\n";
	}
	return *this;
}

const Frame& Frame::write(const std::string path,const std::string& separator) const {
	std::ofstream file(path);
	return write(file,separator);
}

void Frame::resize_(unsigned int rows, unsigned int columns){
	Data::extent_gen extents;
	data_->resize(extents[rows][columns]);
}

void Frame::delta_(int rows, int columns){
	auto shape = data_->shape();
	Data::extent_gen extents;
	data_->resize(extents[shape[0]+rows][shape[1]+columns]);
}

}

std::ostream& operator<<(std::ostream& stream, const Stencila::Frame& frame){
	frame.write(stream);
	return stream;
}
