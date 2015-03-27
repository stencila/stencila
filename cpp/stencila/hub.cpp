#include <boost/network/utils/base64/encode.hpp>

#include <stencila/host.hpp>
#include <stencila/hub.hpp>
#include <stencila/json.hpp>
#include <stencila/http-client.hpp>

namespace Stencila {

Hub hub;

const std::string Hub::root_ = "https://stenci.la/";

Hub& Hub::signin(const std::string& username, const std::string& password){
	Http::Request request(Http::GET,root_+"user/permit/");
	request.auth_basic(username,password);
	
	auto response = client_.request(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	
	return *this;
}

Hub& Hub::signin(const std::string& token){
	Http::Request request(Http::GET,root_+"user/permit/");
	request.header("Authorization","Token "+token);
	
	auto response = client_.request(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	
	return *this;
}

Hub& Hub::signin(void){
	std::string token = Host::variable("STENCILA_TOKEN");
	if(token.length()==0) STENCILA_THROW(Exception,"Environment variable STENCILA_TOKEN is not defined");
	return signin(token);
}

std::string Hub::username(void) const {
	return username_;
}

Hub& Hub::signout(void){
	username_.clear();
	permit_.clear();
	return *this;
}

Hub::Document Hub::request(Http::Method method, const std::string& path){
	std::string url = root_ + path;
	if(path.back()!='/') url += "/";
	Http::Request request(method,url);
	request.header("Authorization","Permit "+permit_);
	auto response = client_.request(request);
	Json::Document doc(response.body());
	return doc;
}

Hub::Document Hub::get(const std::string& path){
	return request(Http::GET,path);
}

Hub::Document Hub::post(const std::string& path){
	return request(Http::POST,path);
}

Hub::Document Hub::delete_(const std::string& path){
	return request(Http::DELETE,path);
}

}
