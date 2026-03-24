#!/usr/bin/env node
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "..");

const tauriConfPath = join(rootDir, "apps/web/src-tauri/tauri.conf.json");
const capabilitiesPath = join(
	rootDir,
	"apps/web/src-tauri/capabilities/default.json"
);
const cargoTomlPath = join(rootDir, "apps/web/src-tauri/Cargo.toml");

const REQUIRED_CSP =
	"default-src 'self'; connect-src 'none'; img-src 'self'; style-src 'self' 'unsafe-inline'";
const FORBIDDEN_PLUGINS = [
	"tauri-plugin-shell",
	"tauri-plugin-http",
	"tauri-plugin-process",
	"tauri-plugin-fs",
];

let errors = 0;

function checkCSP() {
	console.log("Checking CSP...");
	const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf-8"));
	const csp = tauriConf.app?.security?.csp;

	if (!csp) {
		console.error("  FAIL: CSP is not set");
		errors++;
		return;
	}

	if (csp !== REQUIRED_CSP) {
		console.error("  FAIL: CSP does not match required policy");
		console.error(`    Expected: ${REQUIRED_CSP}`);
		console.error(`    Got:      ${csp}`);
		errors++;
		return;
	}

	if (!csp.includes("connect-src 'none'")) {
		console.error("  FAIL: connect-src is not set to 'none'");
		errors++;
		return;
	}

	console.log("  PASS: CSP is correctly configured");
}

function checkForbiddenPlugins() {
	console.log("Checking Cargo.toml for forbidden plugins...");
	const cargoToml = readFileSync(cargoTomlPath, "utf-8");

	for (const plugin of FORBIDDEN_PLUGINS) {
		if (cargoToml.includes(plugin)) {
			console.error(`  FAIL: Forbidden plugin '${plugin}' is included`);
			errors++;
		}
	}

	if (errors === 0 || FORBIDDEN_PLUGINS.every((p) => !cargoToml.includes(p))) {
		console.log("  PASS: No forbidden plugins found");
	}
}

function checkCapabilities() {
	console.log("Checking capabilities...");
	const capabilities = JSON.parse(readFileSync(capabilitiesPath, "utf-8"));
	const permissions = capabilities.permissions || [];

	const dangerousPerms = permissions.filter(
		(p) =>
			p.includes("shell") ||
			p.includes("http") ||
			p.includes("process") ||
			p.includes("fs")
	);

	if (dangerousPerms.length > 0) {
		console.error("  FAIL: Found dangerous permissions:");
		dangerousPerms.forEach((p) => console.error(`    - ${p}`));
		errors++;
	} else {
		console.log("  PASS: Capabilities look safe");
	}
}

console.log("=== Security Hardening Validation ===\n");
checkCSP();
checkForbiddenPlugins();
checkCapabilities();

console.log("\n=== Summary ===");
if (errors > 0) {
	console.error(`FAILED: ${errors} error(s) found`);
	process.exit(1);
} else {
	console.log("PASSED: All security checks passed");
	process.exit(0);
}
