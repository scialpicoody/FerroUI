import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";
import { build, treeDigest, verifyBuild } from "../build-webapp.mjs";
import { CompressionCache } from "../compression-cache.mjs";

const scripts = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const emitted = ["ferroui.js", "ferroui.js.map", "storage.js", "storage.js.map", "ferroui-sw.js", "ferroui-sw.js.map"];

function fixture(t) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "ferroui-build-tools-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    const webapp = path.join(root, "webapp");
    fs.mkdirSync(path.join(webapp, "modules"), { recursive: true });
    fs.writeFileSync(path.join(webapp, "modules/input.ts"), "export const value = 1;\n");
    fs.writeFileSync(path.join(webapp, "package-lock.json"), "{}\n");
    fs.writeFileSync(path.join(webapp, ".eslintrc.json"), "{}\n");
    const calls = [];
    const run = (args) => {
        calls.push(args);
        if (args[0] === "ci") {
            fs.mkdirSync(path.join(webapp, "node_modules/dependency"), { recursive: true });
            fs.writeFileSync(path.join(webapp, "node_modules/dependency/index.js"), "const dependency = 1;\n");
        }
        if (args[1] === "build") {
            fs.mkdirSync(path.join(root, "dist"), { recursive: true });
            for (const name of emitted) fs.writeFileSync(path.join(root, "dist", name), `output ${name}\n`);
        }
    };
    return { root, webapp, run, calls };
}

test("a receipt is written only after the full install, typecheck, lint and bundle recipe", t => {
    const f = fixture(t);
    assert.throws(() => verifyBuild(f.webapp), /absent or stale/);
    build(f.webapp, f.run);
    assert.deepEqual(f.calls, [["ci", "--no-audit", "--no-fund"], ["run", "typecheck"], ["run", "lint"], ["run", "build"]]);
    assert.doesNotThrow(() => verifyBuild(f.webapp));
    const before = f.calls.length;
    verifyBuild(f.webapp);
    assert.equal(f.calls.length, before);
});

test("source edits invalidate even when file length and timestamp are unchanged", t => {
    const f = fixture(t);
    build(f.webapp, f.run);
    const source = path.join(f.webapp, "modules/input.ts");
    const stat = fs.statSync(source);
    fs.writeFileSync(source, "export const value = 2;\n");
    fs.utimesSync(source, stat.atime, stat.mtime);
    assert.throws(() => verifyBuild(f.webapp), /absent or stale/);
});

for (const [name, mutate] of [
    ["new input", f => fs.writeFileSync(path.join(f.webapp, "modules/new.ts"), "export {};\n")],
    ["removed input", f => fs.unlinkSync(path.join(f.webapp, "modules/input.ts"))],
    ["lint configuration", f => fs.writeFileSync(path.join(f.webapp, ".eslintrc.json"), '{"root":true}')],
    ["dependency", f => fs.appendFileSync(path.join(f.webapp, "node_modules/dependency/index.js"), "changed")],
    ["removed dependency", f => fs.unlinkSync(path.join(f.webapp, "node_modules/dependency/index.js"))],
    ["emitted JavaScript", f => fs.appendFileSync(path.join(f.root, "dist/ferroui.js"), "changed")],
    ["source map", f => fs.appendFileSync(path.join(f.root, "dist/ferroui.js.map"), "changed")],
    ["missing service worker", f => fs.unlinkSync(path.join(f.root, "dist/ferroui-sw.js"))],
]) {
    test(`${name} changes cannot reuse a validated build`, t => {
        const f = fixture(t);
        build(f.webapp, f.run);
        mutate(f);
        assert.throws(() => verifyBuild(f.webapp), /absent or stale/);
    });
}

test("a failed gate invalidates an earlier receipt", t => {
    const f = fixture(t);
    build(f.webapp, f.run);
    assert.throws(() => build(f.webapp, args => {
        if (args[1] === "lint") throw new Error("lint failed");
        f.run(args);
    }), /lint failed/);
    assert.throws(() => verifyBuild(f.webapp), /absent or stale/);
});

test("build environment changes invalidate receipts without storing configuration values", t => {
    const f = fixture(t);
    build(f.webapp, f.run);
    const previous = process.env.npm_config_ferroui_fixture;
    try {
        process.env.npm_config_ferroui_fixture = "private-registry-fixture";
        assert.throws(() => verifyBuild(f.webapp), /absent or stale/);
        build(f.webapp, f.run);
        assert.doesNotThrow(() => verifyBuild(f.webapp));
        const receipt = fs.readFileSync(path.join(f.root, "dist/.ferroui-validated-webapp.json"), "utf8");
        assert.ok(!receipt.includes("private-registry-fixture"));
    } finally {
        if (previous === undefined) delete process.env.npm_config_ferroui_fixture;
        else process.env.npm_config_ferroui_fixture = previous;
    }
});

test("changing source during a validation command cannot produce a receipt", t => {
    const f = fixture(t);
    assert.throws(() => build(f.webapp, args => {
        f.run(args);
        if (args[1] === "build") fs.appendFileSync(path.join(f.webapp, "modules/input.ts"), "changed");
    }), /inputs changed during validation/);
    assert.throws(() => verifyBuild(f.webapp), /absent or stale/);
});

test("directory symlink cycles are rejected instead of reusing incomplete input coverage", t => {
    const f = fixture(t);
    if (process.platform === "win32") return t.skip("creating symlinks may require Windows privileges");
    fs.symlinkSync(f.webapp, path.join(f.webapp, "loop"));
    assert.throws(() => treeDigest(f.webapp), /cyclic input/);
});

test("cached lengths equal actual gzip9 and Brotli11 for binary and empty files", async t => {
    const f = fixture(t);
    const directory = path.join(f.root, "cache");
    const inputs = [Buffer.alloc(0), Buffer.from(Array.from({ length: 8192 }, (_, i) => (i * 37) % 256))];
    for (const input of inputs) {
        const expected = [input.length, zlib.gzipSync(input, { level: 9 }).length, zlib.brotliCompressSync(input, {
            params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: input.length },
        }).length];
        const cold = new CompressionCache(directory);
        assert.deepEqual(await cold.measure(input), expected);
        assert.equal(cold.misses, 1);
        const warm = new CompressionCache(directory);
        assert.deepEqual(await warm.measure(input), expected);
        assert.equal(warm.hits, 1);
        assert.equal(warm.misses, 0);
    }
});

test("equal-length changed contents miss the compression cache", async t => {
    const f = fixture(t);
    const directory = path.join(f.root, "cache");
    await new CompressionCache(directory).measure(Buffer.from("aaaaaa"));
    const cache = new CompressionCache(directory);
    const sizes = await cache.measure(Buffer.from("abcdef"));
    assert.equal(cache.hits, 0);
    assert.equal(cache.misses, 1);
    assert.deepEqual(sizes, await new CompressionCache().measure(Buffer.from("abcdef")));
});

test("malformed, foreign-key and inaccessible caches fall back to actual compression", async t => {
    const f = fixture(t);
    const directory = path.join(f.root, "cache");
    const content = Buffer.from("a repeated measurement\n".repeat(100));
    const expected = await new CompressionCache(directory).measure(content);
    const file = path.join(directory, fs.readdirSync(directory)[0]);
    const numericCorruption = JSON.parse(fs.readFileSync(file, "utf8"));
    numericCorruption.sizes[1]++;
    for (const corrupt of ["not JSON", JSON.stringify({ schema: 1, key: "foreign", sizes: [content.length, 0, 0] }), JSON.stringify({ schema: 1, sizes: [0, -1, null] }), JSON.stringify(numericCorruption)]) {
        fs.writeFileSync(file, corrupt);
        const cache = new CompressionCache(directory);
        assert.deepEqual(await cache.measure(content), expected);
        assert.equal(cache.misses, 1);
    }
    const notDirectory = path.join(f.root, "not-directory");
    fs.writeFileSync(notDirectory, "file");
    assert.deepEqual(await new CompressionCache(notDirectory).measure(content), expected);
});

test("simultaneous equal files share one measurement and atomic writers leave usable records", async t => {
    const f = fixture(t);
    const directory = path.join(f.root, "cache");
    const content = Buffer.from("shared contents".repeat(1000));
    const cache = new CompressionCache(directory);
    const sizes = await Promise.all([cache.measure(content), cache.measure(content), new CompressionCache(directory).measure(content)]);
    assert.deepEqual(sizes[0], sizes[1]);
    assert.deepEqual(sizes[0], sizes[2]);
    assert.equal(cache.misses, 1);
    assert.equal(cache.hits, 1);
    assert.ok(fs.readdirSync(directory).every(name => name.endsWith(".json")));
    const restored = new CompressionCache(directory);
    assert.deepEqual(await restored.measure(content), sizes[0]);
    assert.equal(restored.hits, 1);
});

test("cached reports and GitHub outputs are byte-for-byte identical to uncached reports", t => {
    const f = fixture(t);
    const site = path.join(f.root, "site");
    fs.mkdirSync(path.join(site, "assets/Example"), { recursive: true });
    fs.writeFileSync(path.join(site, "module.wasm"), Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]));
    fs.writeFileSync(path.join(site, "module.js"), "export default {};\n");
    fs.writeFileSync(path.join(site, "module.js.map"), "must not count");
    fs.writeFileSync(path.join(site, "assets/Example/hello.txt"), "a fixture asset\n");
    fs.writeFileSync(path.join(site, "assets/Example.json"), JSON.stringify({
        directory: "assets/Example", files: { "/hello.txt": {} }, startup: ["/hello.txt"], pages: { Home: ["/hello.txt"] },
    }));
    function report(name, flags) {
        const githubOutput = path.join(f.root, `${name}.out`);
        const stdoutPath = path.join(f.root, `${name}.stdout`);
        const stderrPath = path.join(f.root, `${name}.stderr`);
        const stdout = fs.openSync(stdoutPath, "w");
        const stderr = fs.openSync(stderrPath, "w");
        let result;
        try {
            result = spawnSync(process.execPath, [path.join(scripts, "module-sizes.mjs"), site, "--github-output", githubOutput, ...flags], { stdio: ["ignore", stdout, stderr] });
        } finally {
            fs.closeSync(stdout);
            fs.closeSync(stderr);
        }
        const errorText = fs.readFileSync(stderrPath, "utf8");
        assert.equal(result.status, 0, errorText);
        return { stdout: fs.readFileSync(stdoutPath, "utf8"), output: fs.readFileSync(githubOutput, "utf8"), stderr: errorText };
    }
    const reference = report("reference", ["--no-cache"]);
    assert.match(reference.stdout, /brotli -11/);
    const cold = report("cold", ["--cache-dir", path.join(f.root, "cache")]);
    const warm = report("warm", ["--cache-dir", path.join(f.root, "cache")]);
    for (const actual of [cold, warm]) {
        assert.equal(actual.stdout, reference.stdout);
        assert.equal(actual.output, reference.output);
    }
    assert.match(warm.stderr, /0 computed/);
    assert.doesNotMatch(warm.stdout, /module\.js\.map/);
});
