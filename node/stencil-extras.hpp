// Extra attributes for stencils

static void attrs_extras_(Local<FunctionTemplate>& tpl) {
  SetPrototypeMethod(tpl, "attach", attach_);
}

static NAN_METHOD(attach_) {
  Isolate* isolate = Isolate::GetCurrent();
  HandleScope scope(isolate);

  Stencil* obj = Unwrap<Stencil>(info.This());
  Stencila::Stencil* inst = static_cast<Stencila::Stencil*>(obj->imp);

  inst->attach(std::make_shared<NodeContext>(info));
}
