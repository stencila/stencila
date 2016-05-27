#include <stencila/component.hpp>

#include <nan.h>

#include "context.hpp"
#include "build/stencil.hpp"
#include "build/sheet.hpp"

void init(v8::Handle<v8::Object> exports) {
  // Register stencila classes
  Stencila::Component::classes();
  // Initialise classes
  Stencil::init_(exports);
  Sheet::init_(exports);
}

NODE_MODULE(extension, init)
