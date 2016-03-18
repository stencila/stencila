#include <iostream>
#include <ctime>

#define STENCILA_DEBUG(message_) \
	std::cerr<<time(0)<<":"<<__FILE__<<":"<<__LINE__<<":"<<message_<<std::endl;
