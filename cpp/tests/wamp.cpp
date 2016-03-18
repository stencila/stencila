#include <boost/test/unit_test.hpp>

#include <stencila/wamp.hpp>

BOOST_AUTO_TEST_SUITE(wamp_quick)

typedef Stencila::Wamp::Message Message;

BOOST_AUTO_TEST_CASE(call){
	Message call(R"([48, 123, {}, "address@method", ["arg1","arg2"], {"kwarg1": 42, "kwarg2": 3.14}])");

	BOOST_CHECK_EQUAL(call.type(), Message::CALL);
	BOOST_CHECK_EQUAL(call.request(), 123);
	BOOST_CHECK_EQUAL(call.procedure(), "address@method");
	BOOST_CHECK_EQUAL(call.procedure_address(), "address");
	BOOST_CHECK_EQUAL(call.procedure_method(), "method");
	BOOST_CHECK_EQUAL(call.args()[0].as<std::string>(), "arg1");
	BOOST_CHECK_EQUAL(call.args()[1].as<std::string>(), "arg2");
	BOOST_CHECK_EQUAL(call.kwargs()["kwarg1"].as<int>(), 42);
	BOOST_CHECK_EQUAL(call.kwargs()["kwarg2"].as<double>(), 3.14);

	auto result = call.result(R"({"a":84})");
	BOOST_CHECK_EQUAL(result.request(), 123);

	auto error = call.error("An error");
	BOOST_CHECK_EQUAL(error[1].as<int>(), Message::CALL);
	BOOST_CHECK_EQUAL(error[2].as<int>(), 123);
}

BOOST_AUTO_TEST_SUITE_END()
