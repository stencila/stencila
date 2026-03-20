# Example: TypeScript — reduce duplication across components

Target files: `src/components/UserCard.tsx`, `src/components/TeamCard.tsx`
Test command: `npm test -- --testPathPattern="(UserCard|TeamCard)"`
Refactoring focus: reduce duplication

**Discovery**: Read sibling components. The codebase uses shared utility components for common UI patterns (e.g., `Avatar`, `Badge`). Both `UserCard.tsx` and `TeamCard.tsx` contain a near-identical 20-line block that renders a card header with an avatar, title, and subtitle, differing only in prop names.

**Analysis**:
- Duplication: identical card-header rendering logic in two components (high impact)
- The shared logic can be extracted to a `CardHeader` component with `avatar`, `title`, and `subtitle` props — consistent with how `Avatar` and `Badge` are used as shared utility components
- Both components' public APIs (their props and exported names) remain unchanged

**Action**:
1. Create `src/components/CardHeader.tsx` with a `CardHeader` component accepting `avatar`, `title`, and `subtitle` props — follow the existing component conventions (explicit prop interface, CSS module, default export)
2. Create `src/components/CardHeader.module.css` with styles extracted from the duplicated blocks
3. Update `UserCard.tsx` to use `<CardHeader>` instead of the inline rendering block
4. Update `TeamCard.tsx` to use `<CardHeader>` instead of the inline rendering block
5. Add `CardHeader` to the barrel export in `src/components/index.ts`
6. Run `npx tsc --noEmit` — clean
7. Run `npm test -- --testPathPattern="(UserCard|TeamCard)"` — all 8 tests pass

**Summary**: Created `src/components/CardHeader.tsx` and `CardHeader.module.css`. Modified `UserCard.tsx` and `TeamCard.tsx` to use the shared component, each replacing a 20-line inline block with a single `<CardHeader>` element. Updated barrel export. Category: duplication. No public API changes to existing components. All tests pass.
