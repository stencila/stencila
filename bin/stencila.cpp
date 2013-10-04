#include <boost/program_options.hpp>

#include <stencila/stencil.hpp>

using namespace Stencila;

void convert(const std::string& input,const std::string& output){
	Stencil stencil("file://"+input);
	stencil.html().write(output,true,false);
}

int main(int argc, const char** argv){
	try {
		namespace po = boost::program_options; 
		po::options_description options_desc("Options"); 
		options_desc.add_options() 
			("help", "Print help")
			("convert", "Convert file from one format to another")
			("input","Input file")
			("output","Output file")
		;

		po::positional_options_description pos_options_desc;
		pos_options_desc.add("input",1);
		pos_options_desc.add("output",1);


		po::variables_map arguments; 
		try { 
			po::store(
				po::command_line_parser(argc, argv).
          			options(options_desc).
         			positional(pos_options_desc).
         			run(),
          		arguments
          	);
			if(arguments.count("help")) { 
				std::cout<< "Stencila" << std::endl << std::endl
						 << options_desc << std::endl; 
				return 0; 
			} 
			po::notify(arguments);
		} 
		catch(po::error& e) { 
			std::cerr << "Error: " << e.what() << std::endl << std::endl
					  << options_desc << std::endl; 
			return 1; 
		} 

		if(arguments.count("convert")) convert(
			arguments["input"].as<std::string>(),
			arguments["output"].as<std::string>()
		);

	}
	catch(std::exception& e){ 
		std::cerr<<"Exiting due to unhandled exception: "<<e.what()<< std::endl; 
		return 2; 
	} 
	return 0;
}