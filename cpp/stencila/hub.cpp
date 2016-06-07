// HACK
// CPR is currently not building on windows so this
// just comments it out. May be fixed or replaced with something
// else
//#include <cpr.h>

#include <stencila/git.hpp>
#include <stencila/host.hpp>
#include <stencila/hub.hpp>
#include <stencila/json.hpp>

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

std::string Hub::url(const std::string& path) const {
	std::string url = origin() + "/" + path;
	if(path.back()!='/') url += "/";
	return url;
}

Hub& Hub::signin(const std::string& username, const std::string& password){
	/*
	Http::Request request(Http::GET,origin()+"/user/permit/");
	request.auth_basic(username,password);
	
	auto response = client_.request(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	*/
	return *this;
}

Hub& Hub::signin(const std::string& token){
	/*
	Host::env_var("STENCILA_TOKEN", token);

	Http::Request request(Http::GET,origin()+"/user/permit/");
	request.header("Authorization","Token "+token);
	
	auto response = client_.request(request);
	Json::Document doc(response.body());
	username_ = doc["username"].as<std::string>();
	permit_ = doc["permit"].as<std::string>();
	*/
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

Json::Document Hub::get(const std::string& path){
	/*
	auto response = cpr::Get(
		cpr::Url{url(path)},
		cpr::Header{
			{"Authorization", "Permit "+permit_},
			{"Accept", "application/json"}
		}
	);
	return Json::Document(response.text);
	*/
	return "";
}

Json::Document Hub::post(const std::string& path){
	/*
	auto response = cpr::Post(
		cpr::Url{url(path)},
		cpr::Header{
			{"Authorization", "Permit "+permit_},
			{"Accept", "application/json"}
		}
	);
	return Json::Document(response.text);
	*/
	return "";
}

Json::Document Hub::delete_(const std::string& path){
	/*
	auto response = cpr::Delete(
		cpr::Url{url(path)},
		cpr::Header{
			{"Authorization", "Permit "+permit_},
			{"Accept", "application/json"}
		}
	);
	return Json::Document(response.text);
	*/
	return "";
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
