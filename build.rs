use std::path::{Path, PathBuf};
use std::process::Command;

const SKIP: &str = "AEGIS_SKIP_WEB_BUILD";

const SOURCE: &str = "web/apps";
const SHARED: &str = "web/shared";
const SCRIPT: &str = "web/build";
const SITE: &str = "web/site";
const DIST: &str = "web/site/dist";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={SOURCE}");
    println!("cargo:rerun-if-changed={SHARED}");
    println!("cargo:rerun-if-changed={SCRIPT}");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=tsconfig.json");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-env-changed={SKIP}");

    for entry in std::fs::read_dir(SITE).into_iter().flatten().flatten() {
        if entry.file_name() != "dist" {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }

    if std::env::var_os("CARGO_FEATURE_WEB").is_none() {
        return;
    }

    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));

    if std::env::var_os(SKIP).is_some() {
        println!("cargo:warning={SKIP} is set.");

        std::fs::create_dir_all(&out).expect("a writable OUT_DIR");

        for name in apps(Path::new(SOURCE)) {
            let page = format!("{name}.html");

            std::fs::write(
                out.join(&page),
                format!(
                    "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\"><title>\
                    Aegis</title></head>\n<body style=\"font-family:sans-serif;max-width:60ch;\
                    margin:60px auto;padding:0 20px\">\n<h1>{page} was not built</h1>\n<p>Built \
                    with <code>{SKIP}</code> set. Rebuild without it.</p>\n</body>\n</html>\n"
                ),
            )
            .unwrap_or_else(|failure| panic!("writing {page}: {failure}"));
        }

        let entries =
            std::fs::read_dir(SOURCE).unwrap_or_else(|failure| panic!("{SOURCE}: {failure}"));

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().unwrap_or_default() == "ts" {
                let name = format!(
                    "{}.js",
                    path.file_stem().unwrap_or_default().to_string_lossy()
                );

                std::fs::write(
                    out.join(&name),
                    format!("throw new Error(\"{name} was not built: {SKIP} was set\");\n"),
                )
                .unwrap_or_else(|failure| panic!("writing {name}: {failure}"));
            }
        }

        table(&out);
        site(&out);

        return;
    }

    pnpm(&["install", "--frozen-lockfile", "--prefer-offline"], None);
    pnpm(&["run", "build"], Some(&out));

    table(&out);
    site(&out);
}

fn apps(from: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(from)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().join("page.html").is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();

    names.sort();

    names
}

fn table(out: &Path) {
    let mut source = String::from("pub static CHUNKS: &[(&str, &str)] = &[\n");

    for app in apps(Path::new(SOURCE)) {
        let mut names: Vec<String> = std::fs::read_dir(out.join(&app))
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .filter(|name| name.ends_with(".js"))
                    .collect()
            })
            .unwrap_or_default();

        names.sort();

        for name in &names {
            let at = format!("{app}/{name}");

            source.push_str(&format!(
                "    ({at:?}, include_str!(concat!(env!(\"OUT_DIR\"), \"/{at}\"))),\n"
            ));
        }
    }

    source.push_str("];\n");

    emit(source.as_bytes(), &out.join("chunks.rs"));
}

fn site(out: &Path) {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets the dir"))
        .join(DIST);

    let mut found = Vec::new();

    walk(&root, &root, &mut found);
    found.sort();

    if found.is_empty() {
        println!("cargo:warning={DIST} is empty. Build it without {SKIP} set.");
    }

    let mut source = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");

    for (route, path) in &found {
        println!("cargo:rerun-if-changed={}", path.display());

        source.push_str(&format!("    ({route:?}, include_bytes!({path:?})),\n"));
    }

    source.push_str("];\n\n");
    source.push_str(&format!("pub const BUILT: bool = {};\n", !found.is_empty()));

    emit(source.as_bytes(), &out.join("site.rs"));
}

fn walk(root: &Path, at: &Path, found: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            walk(root, &path, found);

            continue;
        }

        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };

        let route = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        found.push((route, path));
    }
}

fn emit(wanted: &[u8], target: &Path) {
    if std::fs::read(target).is_ok_and(|existing| existing == wanted) {
        return;
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).expect("a writable OUT_DIR");
    }

    std::fs::write(target, wanted).expect("a writable OUT_DIR");
}

fn pnpm(args: &[&str], out: Option<&Path>) {
    let mut command = match cfg!(windows) {
        true => {
            let mut shell = Command::new("cmd");

            shell.arg("/C").arg("pnpm").args(args);
            shell
        }
        false => {
            let mut direct = Command::new("pnpm");

            direct.args(args);
            direct
        }
    };

    if let Some(out) = out {
        command.env("AEGIS_ASSET_OUT", out);
    }

    let outcome = match command.output() {
        Ok(outcome) => outcome,
        Err(failure) => panic!(
            "could not run pnpm ({failure}). Install it, set {SKIP}=1 to stub the pages, or \
            build without the `web` feature."
        ),
    };

    if !outcome.status.success() {
        panic!(
            "pnpm {} failed:\n{}\n{}",
            args.join(" "),
            String::from_utf8_lossy(&outcome.stdout),
            String::from_utf8_lossy(&outcome.stderr)
        );
    }
}
