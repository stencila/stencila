#include <boost/network/utils/base64/encode.hpp>

#include <stencila/git.hpp>
#include <stencila/host.hpp>
#include <stencila/hub.hpp>
#include <stencila/http-client.hpp>
#include <stencila/json.hpp>

#include <iostream>

namespace Stencila {

Hub hub;

Hub::Hub(void){
	std::string origin = Host::env_var("STENCILA_ORIGIN");
	if(origin.length()){
		origin_ = origin;
	} else {
		origin_ = "https://stenci.la";
	}
}

std::string Hub::origin(void) const {
	return origin_;
}

Hub& Hub::signin(const std::string& username, const std::string& password){
	Http::Request request(Http::GET,origin()+"/user/permit/");
	request.auth_basic(username,password);
	
	auto response = client_.request(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	
	return *this;
}

Hub& Hub::signin(const std::string& token){
	Host::env_var("STENCILA_TOKEN", token);

	Http::Request request(Http::GET,origin()+"/user/permit/");
	request.header("Authorization","Token "+token);
	
	auto response = client_.request(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	
	return *this;
}

Hub& Hub::signin(void){
	return signin(token());
}

std::string Hub::token(void) const {
	std::string token = Host::env_var("STENCILA_TOKEN");
	if(token.length()) return token;
	else return "None";
}

std::string Hub::username(void) const {
	return username_;
}

Hub& Hub::signout(void){
	username_.clear();
	permit_.clear();
	return *this;
}

Json::Document Hub::request(Http::Method method, const std::string& path){
	std::string url = origin() + "/" + path;
	if(path.back()!='/') url += "/";
	Http::Request request(method,url);
	request.header("Authorization","Permit "+permit_);
	auto response = client_.request(request);
	Json::Document doc(response.body());
	return doc;
}

Json::Document Hub::get(const std::string& path){
	return request(Http::GET,path);
}

Json::Document Hub::post(const std::string& path){
	return request(Http::POST,path);
}

Json::Document Hub::delete_(const std::string& path){
	return request(Http::DELETE_,path);
}

std::string Hub::clone(const std::string& address) {
	std::string path = Host::store_path(address);
	Git::Repository repo;
	repo.clone(
		origin() + "/" + address + ".git",
		path
	);
	return path;
}

std::string Hub::fork(const std::string& from, const std::string& to) {
	std::string path = Host::store_path(to);
	Git::Repository repo;
	repo.clone(
		origin() + "/" + from + ".git",
		path
	);
	repo.remote("origin","");
	return path;
}

}
