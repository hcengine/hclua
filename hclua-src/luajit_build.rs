#[allow(dead_code)]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
    options: Options,
}

pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    libs: Vec<String>,
}

#[derive(Default, Clone, Copy)]
struct Options {
    lua52compat: bool,
}

impl Build {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("luajit-build")),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
            options: Options::default(),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    pub fn lua52compat(&mut self, enabled: bool) -> &mut Build {
        self.options.lua52compat = enabled;
        self
    }

    fn cmd_make(&self) -> Command {
        match &self.host.as_ref().expect("HOST dir not set")[..] {
            "x86_64-unknown-dragonfly" => Command::new("gmake"),
            "x86_64-unknown-freebsd" => Command::new("gmake"),
            _ => Command::new("make"),
        }
    }

    pub fn build(&mut self) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET not set")[..];

        if target.contains("msvc") {
            return self.build_msvc();
        }

        self.build_unix()
    }

    pub fn build_unix(&mut self) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET not set")[..];
        let host = &self.host.as_ref().expect("HOST not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("luajit2");
        let build_dir = out_dir.join("build");
        let lib_dir = out_dir.join("lib");
        let include_dir = out_dir.join("include");

        for dir in &[&build_dir, &lib_dir, &include_dir] {
            if dir.exists() {
                fs::remove_dir_all(dir)
                    .unwrap_or_else(|e| panic!("cannot remove {}: {}", dir.display(), e));
            }
            fs::create_dir_all(dir)
                .unwrap_or_else(|e| panic!("cannot create {}: {}", dir.display(), e));
        }
        cp_r(&source_dir, &build_dir);

        let mut cc = cc::Build::new();
        cc.target(target).host(host).warnings(false).opt_level(2);
        let compiler = cc.get_compiler();
        let compiler_path = compiler.path().to_str().unwrap();

        let mut make = self.cmd_make();
        make.current_dir(build_dir.join("src"));
        make.arg("-e");

        match target {
            "x86_64-apple-darwin" if env::var_os("MACOSX_DEPLOYMENT_TARGET").is_none() => {
                make.env("MACOSX_DEPLOYMENT_TARGET", "10.11");
            }
            "aarch64-apple-darwin" if env::var_os("MACOSX_DEPLOYMENT_TARGET").is_none() => {
                make.env("MACOSX_DEPLOYMENT_TARGET", "11.0");
            }
            _ if target.contains("linux") => {
                make.env("TARGET_SYS", "Linux");
            }
            _ if target.contains("windows") => {
                make.env("TARGET_SYS", "Windows");
            }
            _ => {}
        }

        let target_pointer_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();
        if target_pointer_width == "32" && env::var_os("HOST_CC").is_none() {
            // 32-bit cross-compilation?
            let host_cc = cc::Build::new().target(host).get_compiler();
            make.env("HOST_CC", format!("{} -m32", host_cc.path().display()));
        }

        // Infer ar/ranlib tools from cross compilers if the it looks like
        // we're doing something like `foo-gcc` route that to `foo-ranlib`
        // as well.
        let prefix = if compiler_path.ends_with("-gcc") {
            &compiler_path[..compiler_path.len() - 3]
        } else if compiler_path.ends_with("-clang") {
            &compiler_path[..compiler_path.len() - 5]
        } else {
            ""
        };

        let compiler_path =
            which::which(compiler_path).unwrap_or_else(|_| panic!("cannot find {compiler_path}"));
        let bindir = compiler_path.parent().unwrap();
        let compiler_path = compiler_path.to_str().unwrap();
        let compiler_args = compiler.cflags_env();
        let compiler_args = compiler_args.to_str().unwrap();
        if env::var_os("STATIC_CC").is_none() {
            make.env("STATIC_CC", format!("{compiler_path} {compiler_args}"));
        }
        if env::var_os("TARGET_LD").is_none() {
            make.env("TARGET_LD", format!("{compiler_path} {compiler_args}"));
        }

        // Find ar
        if env::var_os("TARGET_AR").is_none() {
            let mut ar = if bindir.join(format!("{prefix}ar")).is_file() {
                bindir.join(format!("{prefix}ar")).into_os_string()
            } else if compiler.is_like_clang() && bindir.join("llvm-ar").is_file() {
                bindir.join("llvm-ar").into_os_string()
            } else if compiler.is_like_gnu() && bindir.join("ar").is_file() {
                bindir.join("ar").into_os_string()
            } else if let Ok(ar) = which::which(format!("{prefix}ar")) {
                ar.into_os_string()
            } else {
                panic!("cannot find {prefix}ar")
            };
            ar.push(" rcus");
            make.env("TARGET_AR", ar);
        }

        // Find strip
        if env::var_os("TARGET_STRIP").is_none() {
            let strip = if bindir.join(format!("{prefix}strip")).is_file() {
                bindir.join(format!("{prefix}strip"))
            } else if compiler.is_like_clang() && bindir.join("llvm-strip").is_file() {
                bindir.join("llvm-strip")
            } else if compiler.is_like_gnu() && bindir.join("strip").is_file() {
                bindir.join("strip")
            } else if let Ok(strip) = which::which(format!("{prefix}strip")) {
                strip
            } else {
                panic!("cannot find {prefix}strip")
            };
            make.env("TARGET_STRIP", strip);
        }

        let mut xcflags = vec!["-fPIC"];
        if self.options.lua52compat {
            xcflags.push("-DLUAJIT_ENABLE_LUA52COMPAT");
        }

        make.env("BUILDMODE", "static");
        make.env("XCFLAGS", xcflags.join(" "));
        self.run_command(make, "building LuaJIT");

        for f in &["lauxlib.h", "lua.h", "luaconf.h", "luajit.h", "lualib.h"] {
            fs::copy(build_dir.join("src").join(f), include_dir.join(f)).unwrap();
        }
        fs::copy(
            build_dir.join("src").join("libluajit.a"),
            lib_dir.join("liblua51.a"),
        )
        .unwrap();

        Artifacts {
            lib_dir,
            include_dir,
            libs: vec!["lua51".to_string()],
        }
    }

    pub fn build_msvc(&mut self) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source_dir = manifest_dir.join("luajit2");
        let extras_dir = source_dir.join("extras");
        let build_dir = out_dir.join("build");
        let lib_dir = out_dir.join("lib");
        let include_dir = out_dir.join("include");

        for dir in &[&build_dir, &lib_dir, &include_dir] {
            if dir.exists() {
                fs::remove_dir_all(dir)
                    .unwrap_or_else(|e| panic!("cannot remove {}: {}", dir.display(), e));
            }
            fs::create_dir_all(dir)
                .unwrap_or_else(|e| panic!("cannot create {}: {}", dir.display(), e));
        }
        cp_r(&source_dir, &build_dir);

        let mut msvcbuild = Command::new(build_dir.join("src").join("msvcbuild.bat"));
        msvcbuild.current_dir(build_dir.join("src"));
        if self.options.lua52compat {
            cp_r(&extras_dir, &build_dir.join("src"));
            msvcbuild.arg("lua52c");
        }
        msvcbuild.arg("static");

        let cl = cc::windows_registry::find_tool(target, "cl.exe").expect("failed to find cl");
        for (k, v) in cl.env() {
            msvcbuild.env(k, v);
        }

        self.run_command(msvcbuild, "building LuaJIT");

        for f in &["lauxlib.h", "lua.h", "luaconf.h", "luajit.h", "lualib.h"] {
            fs::copy(build_dir.join("src").join(f), include_dir.join(f)).unwrap();
        }

        fs::copy(
            build_dir.join("src").join("lua51.lib"),
            lib_dir.join("lua51.lib"),
        ).unwrap();

        Artifacts {
            lib_dir,
            include_dir,
            libs: vec!["lua51".to_string()],
        }
    }

    fn run_command(&self, mut command: Command, desc: &str) {
        let status = command.status().unwrap();
        if !status.success() {
            panic!(
            "
                Error {}:
                Command: {:?}
                Exit status: {}
            ",
                desc, command, status
            );
        }
    }
}

fn cp_r(src: &Path, dst: &Path) {
    for f in fs::read_dir(src).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();

        // Skip git metadata
        if name.to_str() == Some(".git") {
            continue;
        }

        let dst = dst.join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}

impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rerun-if-env-changed=HOST_CC");
        println!("cargo:rerun-if-env-changed=STATIC_CC");
        println!("cargo:rerun-if-env-changed=TARGET_LD");
        println!("cargo:rerun-if-env-changed=TARGET_AR");
        println!("cargo:rerun-if-env-changed=TARGET_STRIP");
        println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}