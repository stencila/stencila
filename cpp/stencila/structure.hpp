#include <stencila/polymorph.hpp>
#include <stencila/mirror-inspect.hpp>
#include <stencila/mirror-rows.hpp>
#include <stencila/mirror-stencil.hpp>

namespace Stencila {

template<class Derived>
class Structure : public Polymorph<Derived>{
public:

  using Polymorph<Derived>::derived;

  typedef Derived structure_type;

  bool has(const std::string& name) {
	  return Mirrors::Has(derived(),name);
  }

  std::vector<std::string> labels(void) {
    return Mirrors::Labels(derived()).result();
  }

  Derived& read(const std::string& path) {
    Stencil stencil;
    stencil.import(path);
    read(stencil);
    return derived();
  }

  Derived& read(Stencil& stencil) {
    Mirrors::StencilParser(derived(),stencil);
    return derived();
  }

  Derived& write(const std::string& path) {
    Stencil stencil;
    write(stencil);
    stencil.export_(path);
    return derived();
  }

  Derived& write(Stencil& stencil) {
    Mirrors::StencilGenerator(derived(),stencil);
    return derived();
  }

  std::string header_row(const std::string& separator="\t") const {
    return Mirrors::RowHeader(derived(),separator);
  }

  std::string to_row(const std::string& separator="\t") {
    return Mirrors::RowGenerator(derived(),separator);
  }

  Derived& from_row(const std::string& row, const std::string& separator="\t") {
    Mirrors::RowParser(derived(),row,separator);
    return derived();
  }
};

} // namespace Stencila
