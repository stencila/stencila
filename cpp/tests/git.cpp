#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/git.hpp>

BOOST_AUTO_TEST_SUITE(git)

using namespace Stencila::Git;

BOOST_AUTO_TEST_CASE(basic){
	Repository repo;
	std::string path = boost::filesystem::unique_path("/tmp/%%%%-%%%%-%%%%-%%%%").string();
	repo.init(path);
	repo.open(path);
	repo.commit("Nokome Bentley","nokome.bentley@stenci.la","The commit message");
	repo.head();
	repo.history();
	repo.destroy();
}

BOOST_AUTO_TEST_SUITE_END()
 