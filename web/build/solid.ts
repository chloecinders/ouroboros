import * as babel from "@babel/core";
import typescript from "@babel/preset-typescript";
import solid from "babel-preset-solid";
import fs from "node:fs/promises";
import type { Plugin } from "esbuild";

export function jsx(generate: "dom" | "ssr"): Plugin {
    return {
        name: "solid-jsx",
        setup(build) {
            build.onLoad({ filter: /\.tsx$/ }, async (loading) => {
                const source = await fs.readFile(loading.path, "utf8");

                const built = await babel.transformAsync(source, {
                    presets: [
                        [solid, { generate, hydratable: false }],
                        [typescript, { isTSX: true, allExtensions: true }],
                    ],
                    filename: loading.path,
                    babelrc: false,
                    configFile: false,
                    sourceMaps: false,
                    compact: false,
                });

                if (!built || built.code === null || built.code === undefined)
                    throw new Error(`babel returned nothing for ${loading.path}`);

                return { contents: built.code, loader: "js" };
            });
        },
    };
}
