/**
 * Template file for a Node.js Stencila class
 */
#pragma once

#include <stencila/{{ name|lower }}.hpp>

#include "extension.hpp"
{% for base in bases %}
#include "{{ base|lower }}.hpp"
{% endfor %}

class {{ name }}:
  {% if name == 'Component' %}public Nan::ObjectWrap{% else %}public {{ bases[0] }}{% endif %}
{
 public:

  {% if name == 'Component' %}
    Stencila::Component* imp = nullptr;
  {% else %}
    {{ name }}(void) {
      imp = new Stencila::{{ name }};
    }
  {% endif %}

  static Nan::Persistent<Function> constructor;

  static NAN_MODULE_INIT(init_) {
    Local<FunctionTemplate> tpl = Nan::New<FunctionTemplate>(new_);
    tpl->SetClassName(Nan::New("{{ name }}").ToLocalChecked());
    tpl->InstanceTemplate()->SetInternalFieldCount(1);

    {% for base in bases %}
      ::{{ base }}::attrs_(tpl);
    {% endfor %}
    {{ name }}::attrs_(tpl);

    constructor.Reset(Nan::GetFunction(tpl).ToLocalChecked());
    Nan::Set(target, Nan::New("{{ name }}").ToLocalChecked(), Nan::GetFunction(tpl).ToLocalChecked());
  }

  static void attrs_(Local<FunctionTemplate>& tpl) {
    {% for attr in attrs %}
      {% if not attr.abstract %}
        SetPrototypeMethod(tpl, "{{ attr.name }}", {{ attr.name }}_);
      {% endif %}
    {% endfor %}
    {{ name }}::attrs_extras_(tpl);
  }

  static NAN_METHOD(new_) {
    if (info.IsConstructCall()) {
      {{ name }}* obj = new {{ name }}();
      obj->Wrap(info.This());
      info.GetReturnValue().Set(info.This());
    } else {
      Local<Function> cons = Nan::New(constructor);
      info.GetReturnValue().Set(cons->NewInstance());
    }
  }

  {% for attr in attrs %}
    {% if not attr.abstract %}

      static NAN_METHOD({{ attr.name }}_) {
        Isolate* isolate = Isolate::GetCurrent();
        HandleScope scope(isolate);

        {{ name }}* obj = Unwrap<{{ name }}>(info.This());
        Stencila::{{ name }}* inst = static_cast<Stencila::{{ name }}*>(obj->imp);

        {% if attr.type == 'method' %}

          inst->{{ attr.name }}();
          {% if attr.return == 'self' %}
            return_self(info);
          {% endif %}

        {% elif attr.type == 'property' %}

          if (info[0]->IsUndefined()) {
            auto value = inst->{{ attr.name }}();
            info.GetReturnValue().Set(
              Nan::New(value).ToLocalChecked()
            );
          } else {
            inst->{{ attr.name }}(
              to<{{ attr.return }}>(info[0])
            );
            return_self(info);
          }

        {% else %}

          inst->{{ attr.name }}();

        {% endif %}
      }

    {% endif %}
  {% endfor %}

  #include "{{ name|lower }}-extras.hpp"
};

Nan::Persistent<Function> {{ name }}::constructor;
