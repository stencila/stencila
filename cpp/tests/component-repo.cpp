#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <stencila/component.hpp>

BOOST_AUTO_TEST_SUITE(component_repo_quick)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(commit){
	Component c;

	BOOST_CHECK_EQUAL(c.commits().size(),0);

	c.commit();
	auto commits = c.commits();
	BOOST_CHECK_EQUAL(commits.size(),2);
	BOOST_CHECK_EQUAL(commits[0].message,"Updated");
	BOOST_CHECK_EQUAL(commits[1].message,"Initial commit");
	BOOST_CHECK(boost::filesystem::exists(c.path()+"/.git"));
	c.destroy();
}

BOOST_AUTO_TEST_CASE(version){
	Component c;
	BOOST_CHECK_EQUAL(c.version(),"");
	c.commit();
	
	BOOST_CHECK_EQUAL(c.version("0.0.1").version(),"0.0.1");
	BOOST_CHECK_THROW(c.version("0.0.0"),Exception);

	BOOST_CHECK_EQUAL(c.version("0.1.0").version(),"0.1.0");
	BOOST_CHECK_THROW(c.version("0.0.1"),Exception);

	BOOST_CHECK_EQUAL(c.version("1.0.0").version(),"1.0.0");
	BOOST_CHECK_THROW(c.version("0.1.0"),Exception);

	c.destroy();
}

BOOST_AUTO_TEST_CASE(provide){
	using boost::filesystem::exists;

	Component c;
	
	c.write_to("version-0.0.1.txt","0.0.1");
		BOOST_CHECK(exists(c.path()+"/version-0.0.1.txt"));
	c.commit();
	c.version("0.0.1");

	c.delete_file("version-0.0.1.txt");
		BOOST_CHECK(not exists(c.path()+"/version-0.0.1.txt"));
	c.write_to("version-0.0.2.txt","0.0.2");
		BOOST_CHECK(exists(c.path()+"/version-0.0.2.txt"));
	c.commit();
	c.version("0.0.2");

	c.provide("0.0.1");
		BOOST_CHECK(exists(c.path()+"/.at/0.0.1/version-0.0.1.txt"));
		BOOST_CHECK(not exists(c.path()+"/.at/0.0.1/version-0.0.2.txt"));
		BOOST_CHECK(not exists(c.path()+"/.at/0.0.1/.git"));

	c.provide("0.0.2");
		BOOST_CHECK(exists(c.path()+"/.at/0.0.2/version-0.0.2.txt"));
		BOOST_CHECK(not exists(c.path()+"/.at/0.0.2/version-0.0.1.txt"));
		BOOST_CHECK(not exists(c.path()+"/.at/0.0.2/.git"));

	c.destroy();
}

BOOST_AUTO_TEST_CASE(get){
	Component c;
	
	c.commit();
	c.version("0.0.1");
	c.version("0.0.2");
	c.hold();

	BOOST_CHECK_EQUAL(c.versions().size(),2);
	BOOST_CHECK_EQUAL(c.versions()[0],"0.0.1");
	BOOST_CHECK_EQUAL(c.versions()[1],"0.0.2");

	Component& c0 = *Component::get(c.address()).as<Component*>();
	BOOST_CHECK(boost::filesystem::exists(c.path()));
	
	Component& c1 = *Component::get(c.address(),"0.0.1").as<Component*>();
	BOOST_CHECK(boost::filesystem::exists(c.path()+"/.at/0.0.1"));

	Component& c2 = *Component::get(c.address(),"0.0.2").as<Component*>();
	BOOST_CHECK(boost::filesystem::exists(c.path()+"/.at/0.0.2"));

	BOOST_CHECK_EQUAL(c0.address(),c1.address());
	BOOST_CHECK_EQUAL(c1.address(),c2.address());

	c.destroy();
}

BOOST_AUTO_TEST_SUITE_END()


BOOST_AUTO_TEST_SUITE(component_repo_slow)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(clone){
	Component::clone("test");
	Component c("test");
	BOOST_CHECK_EQUAL(c.address(),"test");
	BOOST_CHECK_EQUAL(c.origin(),"https://stenci.la/test.git");
	c.destroy();
}

BOOST_AUTO_TEST_CASE(fork){
	Component::fork("test","mytest");
	Component c("mytest");
	BOOST_CHECK_EQUAL(c.address(),"mytest");
	BOOST_CHECK_EQUAL(c.origin(),"");
	c.destroy();
}

BOOST_AUTO_TEST_CASE(get_remote){
	Component& c = *Component::get("test").as<Component*>();
	BOOST_CHECK_EQUAL(c.address(),"test");
	BOOST_CHECK_EQUAL(c.origin(),"https://stenci.la/test.git");
	c.destroy();
}

BOOST_AUTO_TEST_SUITE_END()
 