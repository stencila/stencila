#pragma once

#include <stencila/polymorph.hpp>
#include <stencila/mirror-inspect.hpp>
#include <stencila/mirror-rows.hpp>
#include <stencila/mirror-stencil.hpp>
#include <stencila/mirror-frame.hpp>

namespace Stencila {

template<class Derived>
class Structure : public Polymorph<Derived>{
public:

  using Polymorph<Derived>::derived;

  typedef bool structure_type;

  bool has(const std::string& name) const {
	  return Mirrors::Has(name).mirror<Derived>();
  }

  std::vector<std::string> labels(void) const {
    return Mirrors::Labels().mirror<Derived>();
  }

  Derived& read(const std::string& path) {
    Stencil stencil;
    stencil.import(path);
    read(stencil);
    return derived();
  }

  Derived& read(const Stencil& stencil) {
    Mirrors::StencilParser(stencil).mirror(derived());
    return derived();
  }

  Derived& read(const Frame& frame,const std::vector<std::string>& exclude) {
    Mirrors::FrameReader(frame,exclude).mirror(derived());
    return derived();
  }

  Derived& write(const std::string& path) {
    Stencil stencil;
    write(stencil);
    stencil.export_(path);
    return derived();
  }

  Derived& write(Stencil& stencil) {
    Mirrors::StencilGenerator(stencil).mirror(derived());
    return derived();
  }

  std::string header_row(const std::string& separator="\t") const {
    return Mirrors::RowHeader(separator).mirror<Derived>();
  }

  std::string to_row(const std::string& separator="\t") {
    return Mirrors::RowGenerator(separator).mirror(derived());
  }

  Derived& from_row(const std::string& row, const std::string& separator="\t") {
    Mirrors::RowParser(row,separator).mirror(derived());
    return derived();
  }
};

} // namespace Stencila
