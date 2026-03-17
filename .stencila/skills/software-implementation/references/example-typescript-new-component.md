# Example: TypeScript — new component

Slice: "Phase 1 / Slice 3" — Button component
Acceptance criteria: `Button` component accepts `label`, `onClick`, and `disabled` props; renders a `<button>` element with the label text
Package: `src/components/`

**Test failure output**:
```
FAIL src/components/__tests__/Button.test.tsx
  ● Test suite failed to run
    Cannot find module '../Button' from 'src/components/__tests__/Button.test.tsx'
```

**Discovery**: Read sibling components (`src/components/Input.tsx`, `src/components/Card.tsx`). All use functional components with explicit prop interfaces (e.g., `interface InputProps`). CSS modules are used for styling (`Input.module.css`). The barrel export file `src/components/index.ts` re-exports all components.

**Action**:
1. Read test file — imports `Button` component, expects props `label: string`, `onClick: () => void`, `disabled?: boolean`
2. Tests assert the component renders a `<button>` element with the label as text content
3. Create `src/components/Button.tsx` with `ButtonProps` interface and functional component matching conventions
4. Update barrel export in `src/components/index.ts` to include `Button`
5. Run `npx tsc --noEmit` — clean

**Summary**: Created `src/components/Button.tsx` with `ButtonProps` interface and `Button` functional component. Modified `src/components/index.ts` to add barrel export. Followed existing component conventions: explicit prop interface, CSS module import (created empty `Button.module.css`), default export.
