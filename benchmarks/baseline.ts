const path = "benchmarks/input.json";
const file = Bun.file(path);

const contents = await file.text();

for (let i = 0; i < 1000; i++) {
  JSON.parse(contents);
}

console.log(0);
