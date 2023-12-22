const file = Bun.file(Bun.argv[2]);

const contents = await file.text();

try {
  JSON.parse(contents);
} catch (e) {
  process.stdout.write("1");
  process.exit(1);
}

process.stdout.write("0");
process.exit(0);
