#include <fstream>

#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/git.hpp>
#include <stencila/host.hpp>

BOOST_AUTO_TEST_SUITE(git_slow)

using namespace Stencila::Git;
using namespace Stencila::Host;

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
}

BOOST_AUTO_TEST_CASE(remote){
	Repository repo;
	boost::filesystem::path path = (
		boost::filesystem::temp_directory_path()/
			boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%")
	);
	std::string origin = "https://github.com/stencila/test.git";
	repo.clone(origin,path.string());
	BOOST_CHECK(boost::filesystem::exists(path/".git"));
	BOOST_CHECK_EQUAL(repo.remote(),origin);
}

BOOST_AUTO_TEST_CASE(branches){
	Repository repo;

	// Do an initial commit so that master branch is present
	repo.init(temp_dirname(),true);

	BOOST_CHECK_EQUAL(repo.branch(),"master");

	repo.sprout("test-branch-1");
	{
		BOOST_CHECK_EQUAL(repo.branch(),"test-branch-1");

		auto branches = repo.branches();
		BOOST_CHECK_EQUAL(branches.size(),2);
		BOOST_CHECK_EQUAL(branches[0],"master");
		BOOST_CHECK_EQUAL(branches[1],"test-branch-1");
	}

	repo.sprout("test-branch-2");
	{
		BOOST_CHECK_EQUAL(repo.branch(),"test-branch-2");

		auto branches = repo.branches();
		BOOST_CHECK_EQUAL(branches.size(),3);
		BOOST_CHECK_EQUAL(branches[0],"master");
		BOOST_CHECK_EQUAL(branches[1],"test-branch-1");
		BOOST_CHECK_EQUAL(branches[2],"test-branch-2");
	}

	repo.lop("test-branch-1");
	{
		auto branches = repo.branches();
		BOOST_CHECK_EQUAL(branches.size(),2);
		BOOST_CHECK_EQUAL(branches[0],"master");
		BOOST_CHECK_EQUAL(branches[1],"test-branch-2");
	}

	// Merge second branch into master
	repo.merge("test-branch-2");

	// Switch to master
	repo.branch("master");
	BOOST_CHECK_EQUAL(repo.branch(),"master");
}

BOOST_AUTO_TEST_CASE(archive){
	Repository repo;
	auto repo_dir = temp_dirname();
	auto new_dir = temp_dirname();
	repo.init(repo_dir,true);
	std::ofstream foo(repo_dir+"/foo.txt");
	repo.commit();
	repo.archive("master",new_dir);
	BOOST_CHECK(boost::filesystem::exists(new_dir+"/foo.txt"));
}

BOOST_AUTO_TEST_SUITE_END()
 