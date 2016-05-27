#pragma once

#include <nan.h>
using v8::HandleScope;
using v8::Function;
using v8::FunctionCallbackInfo;
using v8::FunctionTemplate;
using v8::Isolate;
using v8::Local;
using v8::Object;
using v8::String;
using v8::Value;

template<typename Type>
Type to(v8::Local<v8::Value> arg);

template<>
std::string to(v8::Local<v8::Value> arg) {
  return std::string(*v8::String::Utf8Value(arg));
}

void return_self(const Nan::FunctionCallbackInfo<v8::Value>& info) {
  info.GetReturnValue().Set(info.This());
}
