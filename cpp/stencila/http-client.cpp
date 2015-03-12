
#include <boost/network/protocol/http/client.hpp>
#include <boost/network/utils/base64/encode.hpp>

#include <stencila/http-client.hpp>
#include <stencila/exception.hpp>
#include <stencila/string.hpp>

namespace Stencila {
namespace Http {

Request::Request(const std::string& url):
	impl_(new Implementation_(url))
{
	header("User-Agent","Stencila embedded");
}

Request::Request(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers
):
	Request(url)
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

Response Client::get(
	const Request& request
){
	auto response = impl_->get(*request.impl_);
	check_(response);
	return Response(response);
}

Response Client::get(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers
){
	return get(Request(url,params,headers));
}

Response Client::post(
	const Request& request
){
	auto response = impl_->post(*request.impl_);
	check_(response);
	return Response(response);
}

Response Client::post(
	const std::string& url,
	const std::map<std::string,std::string>& params,
	const std::map<std::string,std::string>& headers,
	const std::string& body
){
	auto request = Request(url,params,headers);
	if(body.length()) request.body(body);
	return post(request);
}

void Client::check_(boost::network::http::client::response& response){
	using namespace boost::network::http;
	int code = status(response);
	if(code>299){
		STENCILA_THROW(
			Exception,
			"Server responded with HTTP non-success HTTP code.\n  code: "+string(code)
		);
	}
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
