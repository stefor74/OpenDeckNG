/// <reference lib="deno.ns" />
// deno-lint-ignore-file no-import-prefix

import { copy } from "jsr:@std/fs@^1.0";
import { join } from "jsr:@std/path@^1.0";

const pluginsDir = "plugins";
const outDir = "target/plugins";

// Detect target triple
const target = await (async () => {
	const output = await new Deno.Command("rustc", { args: ["--version", "--verbose"] }).output();
	const text = new TextDecoder().decode(output.stdout);
	const match = text.match(/host: ([^\n]+)/);
	return match ? match[1] : "x86_64-unknown-linux-gnu";
})();

console.log(`Building plugins for target: ${target}`);

for await (const entry of Deno.readDir(pluginsDir)) {
	if (!entry.isDirectory || !entry.name.endsWith(".sdPlugin")) continue;

	const pluginPath = join(pluginsDir, entry.name);
	const buildScript = join(pluginPath, "build.ts");

	try {
		await Deno.stat(buildScript);
	} catch {
		console.warn(`No build.ts found for ${entry.name}, skipping`);
		continue;
	}

	const pluginOutDir = join(outDir, entry.name);
	console.log(`Building ${entry.name}...`);

	const status = await new Deno.Command("deno", {
		args: ["run", "--allow-all", buildScript, pluginOutDir, target],
		cwd: Deno.cwd(),
	}).spawn().status;

	if (!status.success) {
		console.error(`Failed to build ${entry.name}`);
		Deno.exit(1);
	}

	console.log(`Built ${entry.name} -> ${pluginOutDir}`);
}

console.log("All plugins built successfully.");
