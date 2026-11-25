import * as fs from "fs";
import * as path from "path";
import { linterCheckTest } from "tests";
import { QltyDriver, SETUP_TIMEOUT_MS } from "tests/driver";
import { getKnownGoodVersion } from "tests/runLinterTest";

linterCheckTest("gitleaks", __dirname);

describe("linter=gitleaks invocation output", () => {
  const driver = new QltyDriver(
    "gitleaks",
    getKnownGoodVersion(__dirname, "gitleaks"),
  );
  const secret = fs
    .readFileSync(path.join(__dirname, "fixtures", "basic.in.py"), "utf-8")
    .split('"')[1];
  const outDir = () => path.join(driver.sandboxPath, ".qlty", "out");
  const storedInvocations = () =>
    fs
      .readdirSync(outDir())
      .map((file) => fs.readFileSync(path.join(outDir(), file), "utf-8"))
      .join("\n");

  beforeAll(async () => {
    await driver.setUp("basic.in.py");
  }, SETUP_TIMEOUT_MS);

  beforeEach(() => {
    fs.rmSync(outDir(), { recursive: true, force: true });
  });

  afterEach(() => {
    delete process.env.QLTY_INSECURE_DISABLE_SECRET_REDACTION;
  });

  afterAll(() => {
    driver.tearDown();
  });

  test("does not store the secret", async () => {
    const result = await driver.runCheck();

    expect(result.runResult.outputJson).toHaveLength(1);
    expect(storedInvocations()).toContain("tmpfileContents");
    expect(storedInvocations()).not.toContain(secret);
    expect(result.runResult.stderr).not.toContain(secret);
  });

  test("stores the secret when redaction is disabled", async () => {
    process.env.QLTY_INSECURE_DISABLE_SECRET_REDACTION = "1";

    const result = await driver.runCheck();

    expect(result.runResult.outputJson).toHaveLength(1);
    expect(storedInvocations()).toContain(secret);
    expect(result.runResult.stderr).toContain(
      "QLTY_INSECURE_DISABLE_SECRET_REDACTION is set",
    );
  });
});
