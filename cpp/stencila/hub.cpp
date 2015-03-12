#include <boost/network/utils/base64/encode.hpp>

#include <stencila/hub.hpp>
#include <stencila/json.hpp>
#include <stencila/http-client.hpp>

namespace Stencila {

Hub hub;

const std::string Hub::root_ = "http://127.0.0.1:8000/";

Hub& Hub::signin(const std::string& username, const std::string& password){
	Http::Request request(root_+"user/permit/");
	request.auth_basic(username,password);
	
	auto response = client_.get(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	
	return *this;
}

Hub& Hub::signin(const std::string& token){
	Http::Request request(root_+"user/permit/");
	request.header("Authorization","Token "+boost::network::utils::base64::encode<char>(token));
	
	auto response = client_.get(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	
	return *this;
}

std::string Hub::username(void) const {
	return username_;
}

Hub& Hub::signout(void){
	username_.clear();
	permit_.clear();
	return *this;
}

Hub::Document Hub::get(const std::string& path){
	std::string url = root_ + path;
	if(path.back()!='/') url += "/";
	Http::Request request(url);
	request.header("Authorization","Permit "+permit_);
	auto response = client_.get(request);
	Json::Document doc(response.body());
	return doc;
}

Hub::Document Hub::post(const std::string& path){
	std::string url = root_ + path;
	if(path.back()!='/') url += "/";
	Http::Request request(url);
	request.header("Authorization","Permit "+permit_);
	auto response = client_.post(request);
	Json::Document doc(response.body());
	return doc;
}

}
