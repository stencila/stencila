#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/git.hpp>

BOOST_AUTO_TEST_SUITE(git)

using namespace Stencila::Git;

BOOST_AUTO_TEST_CASE(basic){
	Repository repo;
	std::string path = (
		boost::filesystem::temp_directory_path()/
			boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%")
	).string();
	repo.init(path);
	repo.open(path);
	repo.commit("Nokome Bentley","nokome.bentley@stenci.la","The commit message");
	repo.head();
	repo.commits();
	repo.destroy();
}

BOOST_AUTO_TEST_CASE(clone_remote){
	Repository repo;
	boost::filesystem::path path = (
		boost::filesystem::temp_directory_path()/
			boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%")
	);
	std::string origin = "https://github.com/stencila/test.git";
	repo.clone(origin,path.string());
	BOOST_CHECK(boost::filesystem::exists(path/".git"));
	BOOST_CHECK_EQUAL(repo.remote(),origin);
	repo.destroy();
}

BOOST_AUTO_TEST_SUITE_END()
 