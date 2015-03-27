#define BOOST_NETWORK_ENABLE_HTTPS
#include <boost/network/protocol/http/client.hpp>
#include <boost/network/utils/base64/encode.hpp>

#include <stencila/http-client.hpp>
#include <stencila/exception.hpp>
#include <stencila/string.hpp>

namespace Stencila {
namespace Http {

Request::Request(Method method, const std::string& url):
	method_(method),
	impl_(new Implementation_(url))
{
	header("User-Agent","Stencila embedded");
}

Request::Request(const std::string& url):
	Request(GET,url)
{}

Request::Request(
	Method method,
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers
):
	Request(method,url)
{
	for(auto item : headers) header(item.first,item.second);
}

Request::~Request(void){
	delete impl_;
}

Request& Request::param(const std::string& name,const std::string& value){
	return *this;
}

Request& Request::header(const std::string& name,const std::string& value){
	*impl_ << boost::network::header(name,value);
	return *this;
}

Request& Request::auth_basic(const std::string& username,const std::string& password){
	std::string encoded = boost::network::utils::base64::encode<char>(
		username+":"+password
	);
	header("Authorization","Basic "+encoded);
	return *this;
}

Request& Request::body(const std::string& body){
	boost::network::body(*impl_,body);
	return *this;
}

Response::Response(const Implementation_& impl):
	impl_(new Implementation_(impl)) {
}

Response::~Response(void){
	delete impl_;
}

int Response::status(void) const {
	return boost::network::http::status(*impl_);
}

std::vector<std::string> Response::headers(const std::string& name) const {
	std::vector<std::string> vec;
	for(auto header : boost::network::http::headers(*impl_)[name]){
		vec.push_back(header.second);
	}
	return vec;
}

std::string Response::cookie(const std::string& name) const {
	auto cookies = boost::network::http::headers(*impl_)["Set-Cookie"];
	for(auto cookie : cookies){
		auto pairs = split(cookie.second,";");
		for(auto pair : pairs){
			if(pair.substr(0, name.length()+1) == (name + '=')){
				return pair.substr(name.length()+1);
			}
		}
	}
	return "";
}

std::string Response::body(void) const {
	return boost::network::http::body(*impl_);
}

Client::Client(void):
	impl_(new Implementation_){
}

Client::~Client(void){
	delete impl_;
}

Response Client::request(const Request& request){
	using namespace boost::network;
	http::response response;
	switch(request.method_){
		case GET:
			response = impl_->get(*request.impl_);
		break;
		case POST:
			response = impl_->post(*request.impl_);
		break;
		case DELETE:
			response = impl_->delete_(*request.impl_);
		break;
		default:
			STENCILA_THROW(Exception,"Request method not currently handled");
		break;
	}
	int code = http::status(response);
	if(code>299){
		STENCILA_THROW(
			Exception,
			"Server responded with a HTTP failure code.\n  code: "+string(code)
		);
	}
	return Response(response);
}

Response Client::get(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers
){
	return this->request(Request(GET,url,params,headers));
}

Response Client::post(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers,
	const std::string& body
){
	auto request = Request(POST,url,params,headers);
	if(body.length()) request.body(body);
	return this->request(request);
}

Response get(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers
){
	Client client;
	return client.get(url,params,headers);
}

Response post(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers,
	const std::string& body
){
	Client client;
	return client.post(url,params,headers,body);
}

}
}
