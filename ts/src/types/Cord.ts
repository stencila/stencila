export type Cord = {
  string: string;
  authorship?: number[][];
  type?: "Cord";
};

/**
 * Create a new `Cord`
 */
export function cord(string: string): Cord {
  return {
    string
  }
}
