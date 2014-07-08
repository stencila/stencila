#include <stencila/polymorph.hpp>

namespace Stencila {

template<class Derived>
class Reflector : public Polymorph<Derived>{
public:

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
		return Has<Derived>(name);
    }
#if 0
    std::string repr(void) const{
          Repr repr();
          repr.mirror(*static_cast<Derived*>(this));
          return repr.repr();
    }
#endif
};

}  // namespace Stencila
