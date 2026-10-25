// Build the shared browser script once, or verify the exact successful build used by a batch.
//
//   node scripts/browser/build-webapp.mjs build
//   node scripts/browser/build-webapp.mjs verify
//
// `verify` is an explicit batch contract, not an implicit npm cache. It refuses changes to any
// source, installed dependency or emitted file. Ordinary build-browser.sh calls still use build.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const repository = path.resolve(path.dirname(scriptPath), "../..");
const webapp = path.join(repository, "src/Browser/FerroUI.Browser/webapp");
const receiptName = ".ferroui-validated-webapp.json";
const outputs = ["ferroui.js", "ferroui.js.map", "storage.js", "storage.js.map", "ferroui-sw.js", "ferroui-sw.js.map"];
const schema = 1;

/** Hash names and contents, including additions/deletions and targets of file symlinks. */
export function treeDigest(directory, excluded = new Set()) {
    const hash = crypto.createHash("sha256");
    const ancestors = new Set();
    function visit(absolute, relative) {
        const link = fs.lstatSync(absolute);
        if (link.isSymbolicLink()) hash.update(`link\0${relative}\0${fs.readlinkSync(absolute)}\0`);
        const stat = fs.statSync(absolute);
        if (stat.isDirectory()) {
            const real = fs.realpathSync(absolute);
            if (ancestors.has(real)) throw new Error(`cyclic input directory: ${relative}`);
            ancestors.add(real);
            hash.update(`directory\0${relative}\0`);
            for (const name of fs.readdirSync(absolute).sort()) {
                if (!relative && excluded.has(name)) continue;
                visit(path.join(absolute, name), relative ? `${relative}/${name}` : name);
            }
            ancestors.delete(real);
        } else if (stat.isFile()) {
            const bytes = fs.readFileSync(absolute);
            hash.update(`file\0${relative}\0${stat.mode & 0o777}\0${bytes.length}\0`);
            hash.update(bytes);
        } else {
            throw new Error(`unsupported input file: ${relative}`);
        }
    }
    visit(directory, "");
    return hash.digest("hex");
}

function identity(directory) {
    // Values may include private registry configuration. Record only their digest, never the
    // configuration itself. Rust target/profile changes do not affect this shared JS bundle.
    const environment = Object.entries(process.env).filter(([name]) =>
        ["NODE_OPTIONS", "NODE_PATH", "NODE_ENV", "ESBUILD_BINARY_PATH", "ESLINT_USE_FLAT_CONFIG"].includes(name)
        || /^npm_config_/i.test(name)).sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0);
    return {
        schema,
        directory: fs.realpathSync(directory),
        node: process.version,
        platform: process.platform,
        arch: process.arch,
        environment: crypto.createHash("sha256").update(JSON.stringify(environment)).digest("hex"),
        // Changing the validation recipe invalidates an old receipt as well.
        recipe: crypto.createHash("sha256").update(fs.readFileSync(scriptPath)).digest("hex"),
        sources: treeDigest(directory, new Set(["node_modules"])),
        dependencies: treeDigest(path.join(directory, "node_modules")),
    };
}

function outputDigest(directory) {
    const dist = path.resolve(directory, "../dist");
    for (const name of outputs) {
        if (!fs.statSync(path.join(dist, name)).isFile()) throw new Error(`missing script output: ${name}`);
    }
    return treeDigest(dist, new Set([receiptName]));
}

/** Write only after every command succeeded; tests can exercise this contract on a fixture. */
export function recordBuild(directory, validatedIdentity) {
    const currentIdentity = identity(directory);
    if (JSON.stringify(currentIdentity) !== JSON.stringify(validatedIdentity)) {
        throw new Error("browser script inputs changed during validation; build again");
    }
    const receipt = path.resolve(directory, "../dist", receiptName);
    const temporary = `${receipt}.${process.pid}.${crypto.randomUUID()}.tmp`;
    const data = { identity: currentIdentity, outputs: outputDigest(directory) };
    try {
        fs.writeFileSync(temporary, `${JSON.stringify(data)}\n`, { flag: "wx" });
        fs.renameSync(temporary, receipt);
    } finally {
        fs.rmSync(temporary, { force: true });
    }
}

export function verifyBuild(directory = webapp) {
    try {
        const receipt = JSON.parse(fs.readFileSync(path.resolve(directory, "../dist", receiptName), "utf8"));
        if (JSON.stringify(receipt.identity) !== JSON.stringify(identity(directory)) || receipt.outputs !== outputDigest(directory)) {
            throw new Error("an input or output changed");
        }
    } catch (error) {
        throw new Error(`the prebuilt browser script is absent or stale (${error.message}); run node scripts/browser/build-webapp.mjs build`);
    }
}

/** Full validation always installs from the lockfile. No successful receipt survives a failure. */
export function build(directory = webapp, run = runNpm) {
    fs.rmSync(path.resolve(directory, "../dist", receiptName), { force: true });
    const beforeInstall = treeDigest(directory, new Set(["node_modules"]));
    run(["ci", "--no-audit", "--no-fund"], directory);
    const validated = identity(directory);
    if (beforeInstall !== validated.sources) throw new Error("browser script sources changed during installation; build again");
    for (const command of ["typecheck", "lint", "build"]) run(["run", command], directory);
    recordBuild(directory, validated);
}

function runNpm(args, directory) {
    const npm = process.platform === "win32" ? "npm.cmd" : "npm";
    // These command names are fixed above; no shell text comes from a document or a caller.
    const result = spawnSync(npm, args, { cwd: directory, stdio: "inherit", shell: process.platform === "win32" });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`npm ${args.join(" ")} failed (${result.signal ?? result.status})`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === scriptPath) {
    try {
        if (process.argv.length !== 3 || !["build", "verify"].includes(process.argv[2])) {
            throw new Error("usage: node scripts/browser/build-webapp.mjs <build|verify>");
        }
        if (process.argv[2] === "build") build();
        else verifyBuild();
        console.log(`Browser script ${process.argv[2] === "build" ? "built and validated" : "verified: exact validated inputs and outputs"}.`);
    } catch (error) {
        console.error(error.message);
        process.exitCode = 1;
    }
}
