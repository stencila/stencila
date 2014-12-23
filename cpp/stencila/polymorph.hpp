#pragma once

namespace Stencila {

/**
 * Base class for static polymorphism
 *
 * This trivial base class is intended to provide for consistent usage of 
 * the [curiously recurring template pattern](http://en.wikipedia.org/wiki/Curiously_recurring_template_pattern)
 * for implementation of static polymorphism.
 */
template<
	class Derived
>
class Polymorph {
public:
	/**
	 * Shortcut method which returns the derived type.
	 *
	 * This can be used to ensure the correct method is called e.g.
	 * 	derived().method();
	 */
	Derived& derived(void) {
		return static_cast<Derived&>(*this);
	}
	const Derived& derived(void) const {
		return static_cast<const Derived&>(*this);
	}
	/**
	 * Create a null pointer to the derived type
	 *
	 * This can be useful when an instance of a type is expensive to construct
	 * but is required as a type indicator for a function
	 */
	static const Derived* derived_nullptr(void) {
		return static_cast<const Derived*>(nullptr);
	}
}; //class Polymorph

} // namespace Stencila
