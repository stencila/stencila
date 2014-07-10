#include <stencila/polymorph.hpp>
#include <stencila/mirrors.hpp>

namespace Stencila {

template<class Derived>
class Reflector : public Polymorph<Derived>{
public:

  using Polymorph<Derived>::derived;

#if 0
    std::string type(void) {
          Type type;
          type.mirror(*static_cast<Derived*>(this));
          return type.type();
    }

    std::vector<std::string> attrs(void) {
          Keys keys;
          keys.mirror(*static_cast<Derived*>(this));
          return keys.keys();
    }
#endif
    bool has(const std::string& name) {
		  return Has(derived(),name);
    }
#if 0
    std::string repr(void) const{
          Repr repr();
          repr.mirror(*static_cast<Derived*>(this));
          return repr.repr();
    }
#endif

    std::string header_row(const std::string& separator="\t") const {
      return RowHeader(derived(),separator);
    }

    std::string to_row(const std::string& separator="\t") {
      return RowGenerator(derived(),separator);
    }

    Derived& from_row(const std::string& row, const std::string& separator="\t") {
      RowParser(derived(),row,separator);
      return derived();
    }
};

} // namespace Stencila
