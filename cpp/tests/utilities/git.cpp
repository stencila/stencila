#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/utilities/git.hpp>

BOOST_AUTO_TEST_SUITE(git)

using namespace Stencila::Utilities::Git;

BOOST_AUTO_TEST_CASE(foo1){
	Repository repo;
	repo.init("outputs/components/1/");
	repo.open("outputs/components/1/");
	repo.make("1.html");
	repo.make("2.html","3.html");
	repo.commit("Nokome Bentley","nokome.bentley@stenci.la","The commit message");
	std::cout<<repo.head()<<std::endl;
	repo.log();
	repo.destroy();
}

BOOST_AUTO_TEST_CASE(foo2){
	Repository repo;
	repo.clone("https://github.com/nokome/ubuntu-bits.git","outputs/components/2/");
}

BOOST_AUTO_TEST_SUITE_END()
 