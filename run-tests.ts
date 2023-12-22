import { readdir } from "node:fs/promises";
import { join } from "node:path";

const args = Bun.argv;
if (args.length != 3) {
  console.log("Usage: bun run-tests.ts <path>");
  process.exit(1);
}

const programPath = args[2];

const results = {
  test_parsing: 0,
  test_transform: 0,
};

const total_tests = {
  test_parsing: 0,
  test_transform: 0,
};

for (let dir_name in results) {
  const testParsingDir = join("tests", dir_name);
  const testParsingFiles = await readdir(testParsingDir);
  console.log(`Running ${dir_name} tests...`);

  for (const test of testParsingFiles) {
    total_tests[dir_name as keyof typeof total_tests]++;
    const filePrefix = test[0];
    process.stdout.write(`Running test: ${test} `);
    const proc = Bun.spawn([programPath, join(testParsingDir, test)]);
    const result = await new Response(proc.stdout).text();
    if (filePrefix === "i") {
      // result doesn't matter as long as it doesn't crash
      results[dir_name as keyof typeof results]++;
    } else if (
      (filePrefix === "y" && result === "0") ||
      (filePrefix === "n" && result === "1")
    ) {
      results[dir_name as keyof typeof results]++;
      process.stdout.write("\x1b[32mOK\x1b[0m\n");
    } else {
      process.stdout.write("\x1b[31mFAILED\x1b[0m\n");
    }
  }
}

console.log(); // newline
const total_test_passed = results.test_parsing + results.test_transform;
console.log(
  `Test parsing: ${results.test_parsing}/${total_tests.test_parsing}`
);
console.log(
  `Test transform: ${results.test_transform}/${total_tests.test_transform}`
);
console.log(
  `Total: ${total_test_passed}/${
    total_tests.test_parsing + total_tests.test_transform
  }`
);
