#include <iostream>
#include <cstdlib>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/compress.hpp>

BOOST_AUTO_TEST_SUITE(compress)

using namespace Stencila::Compress;

BOOST_AUTO_TEST_CASE(strings){
    Writer w("outputs/compress-strings.tar.gz");
    w.set("id/1.txt","Hello1");
    w.set("id/2.txt","Hello2");
    w.close();
    
    Reader r("outputs/compress-strings.tar.gz");
    BOOST_CHECK_EQUAL(r.get("id/1.txt"),"Hello1");
    BOOST_CHECK_EQUAL(r.get("id/2.txt"),"Hello2");
    BOOST_CHECK_EQUAL(r.get("id/some-non-existant-path.txt"),"");
}

BOOST_AUTO_TEST_CASE(files){
    std::ofstream file("outputs/compress-files-1.txt");
    file<<"654321";
    for(int i=0;i<100000;i++) file<<std::rand();
    file.close();
    
    Writer w("outputs/compress-files.tar.gz");
    w.add("id/a","outputs/compress-files-1.txt");
    w.close();
    
    Reader r("outputs/compress-files.tar.gz");
    r.extract("id/a","outputs/compress-files-2");
}

BOOST_AUTO_TEST_SUITE_END()