# Example: TypeScript package with no existing tests

Slice: "Phase 1 / Slice 1" — URL validation utility
Acceptance criteria: `validateUrl` returns true for valid HTTP/HTTPS URLs; returns false for empty strings, non-URL strings, and FTP URLs
Package: `lib/`

**Discovery**: The `package.json` has `vitest` as a dev-dependency but no test files exist yet. No `__tests__/` directory or `*.test.ts` files are found anywhere in the project. The `tsconfig.json` indicates TypeScript.

**Fallback applied**: Since no test conventions are present, the fallback table is used — TypeScript with Vitest. Tests are placed in `lib/__tests__/validate-url.test.ts` following Vitest conventions.

Test written in `lib/__tests__/validate-url.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import { validateUrl } from "../validate-url";

describe("validateUrl", () => {
  it("returns true for valid HTTP URL", () => {
    expect(validateUrl("http://example.com")).toBe(true);
  });

  it("returns true for valid HTTPS URL", () => {
    expect(validateUrl("https://example.com/path?q=1")).toBe(true);
  });

  it("returns false for empty string", () => {
    expect(validateUrl("")).toBe(false);
  });

  it("returns false for non-URL string", () => {
    expect(validateUrl("not a url")).toBe(false);
  });

  it("returns false for FTP URL", () => {
    expect(validateUrl("ftp://files.example.com")).toBe(false);
  });
});
```

Context stored:

- `slice.test_files` = `lib/__tests__/validate-url.test.ts`
- `slice.test_command` = `npx vitest run lib/__tests__/validate-url.test.ts`

Summary notes: "No existing tests found in the project. Applied fallback conventions for TypeScript with Vitest based on the dev-dependency in package.json."
