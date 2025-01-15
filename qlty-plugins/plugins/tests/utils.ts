export type LinterVersion =
  | "KnownGoodVersion"
  | "Latest"
  | "Snapshots"
  | string;

export interface EnvOptions {
  /** Version of linters to enable and test against. */
  linterVersion?: LinterVersion | string;

  /** Prevents the deletion of sandbox test dirs. */
  sandboxDebug: boolean;

  /** Test the output against the known good version output of the linter. */
  testAgainstKnownGoodVersion: boolean;
}

const parseLinterVersion = (value: string): LinterVersion | undefined => {
  if (value && value.length > 0) {
    return value;
  }
  return undefined;
};

export const OPTIONS: EnvOptions = {
  linterVersion: parseLinterVersion(
    process.env.QLTY_PLUGINS_LINTER_VERSION ?? "",
  ),
  sandboxDebug: Boolean(process.env.QLTY_PLUGINS_SANDBOX_DEBUG),
  testAgainstKnownGoodVersion: Boolean(
    process.env.QLTY_PLUGINS_TEST_AGAINST_KNOWN_GOOD_VERSION,
  ),
};

const extractStructure = (obj: object): Record<string, unknown> => {
  const structure: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === "object" && value !== null) {
      if (Array.isArray(value)) {
        structure[key] = "array";
      } else {
        structure[key] = extractStructure(value); // Recursive call for nested objects
      }
    } else {
      structure[key] = typeof value; // Store the type of the value
    }
  }
  return structure;
};

export const serializeStructure = (obj: object) => {
  const structure = extractStructure(obj);
  return JSON.stringify(structure, undefined, 2);
};
