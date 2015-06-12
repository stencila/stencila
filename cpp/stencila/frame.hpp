#pragma once

#include <string>
#include <vector>
#include <limits>

namespace boost {
	template<typename T, std::size_t NumDims,typename Allocator> class multi_array;
}

namespace Stencila {

class Frame {
public:

	Frame(void);

	Frame(const Frame& frame);

	Frame(const std::vector<std::string>& labels, unsigned int rows=0);

	Frame(unsigned int rows, const std::vector<std::string>& labels={});

	Frame(const std::vector<std::string>& labels, const std::vector<double>& values);

	~Frame(void);

	template<class Structure>
	static Frame of(void);


	unsigned int rows(void) const;

	unsigned int columns(void) const;

	bool empty(void) const;


	std::vector<std::string> labels(void) const;

	std::string label(unsigned int index) const;

	int label(const std::string& label) const;

	bool has(const std::string& label) const;


	double& operator()(unsigned int row, unsigned int column);
	const double& operator()(unsigned int row, unsigned int column) const;

	double& operator()(unsigned int row, const std::string& label);
	const double& operator()(unsigned int row, const std::string& label) const;


	std::vector<double> row(unsigned int row) const;

	std::vector<double> column(unsigned int column) const;

	std::vector<double> column(const std::string& label) const;

	Frame slice(unsigned int row) const;

	Frame slice(unsigned int from,unsigned int to) const;

	Frame chop(unsigned int from) const;

	Frame chop(unsigned int from,unsigned int to) const;

	Frame chop(const std::vector<double>& labels) const;

	Frame dice(unsigned int row_from,unsigned int row_to,unsigned int col_from, unsigned int col_to) const;


	Frame& add(const std::string& label,const double& value = 0);


	Frame& append(unsigned int rows = 1);

	Frame& append(const std::vector<double>& row);

	Frame& append(const std::vector<std::string>& row);

	Frame& append(const Frame& frame);

	Frame& clear(void);


	Frame& read(std::istream& stream, const std::string& separator=" \t");

	Frame& read(const std::string path, const std::string& separator=" \t");

	const Frame& write(std::ostream& stream, const std::string& separator="\t") const;

	const Frame& write(const std::string path, const std::string& separator="\t") const;

private:

	void resize_(unsigned int rows, unsigned int columns);

	void delta_(int rows, int columns);

	typedef boost::multi_array<double,2,std::allocator<double>> Data;
	Data* data_;
	std::vector<std::string> labels_;
};

template<class Structure>
Frame Frame::of(void){
	return Frame(static_cast<Structure*>(nullptr)->labels());
}

}

/**
 * Output a Frame to a stream using the `<<` operator
 */
std::ostream& operator<<(std::ostream& stream, const Stencila::Frame& frame);


#if defined(STENCILA_INLINE) && !defined(STENCILA_FRAME_CPP)
#include <stencila/frame.cpp>
#endif
