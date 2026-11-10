mod registry;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use registry::{registry, Kind, Lib};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

// ───────────────────────────── Manifesto ─────────────────────────────

#[derive(Deserialize)]
struct Manifest {
    package: Package,
    #[serde(default)]
    dependencies: BTreeMap<String, DepSpec>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    #[serde(default = "d_version")]
    #[allow(dead_code)]
    version: String,
    #[serde(rename = "type", default = "d_type")]
    kind: String, // exe | static | shared
    #[serde(default = "d_std")]
    std: u32,
    #[serde(default = "d_sources")]
    sources: Vec<String>,
    #[serde(default = "d_include")]
    include: Vec<String>,
    #[serde(default)]
    defines: Vec<String>,
    #[serde(default)]
    cxxflags: Vec<String>,
    #[serde(default)]
    libs: Vec<String>,
    assets: Option<String>,
}

fn d_version() -> String { "0.1.0".into() }
fn d_type() -> String { "exe".into() }
fn d_std() -> u32 { 20 }
fn d_sources() -> Vec<String> { vec!["src/*.cpp".into(), "src/*.c".into(), "src/*.cc".into()] }
fn d_include() -> Vec<String> { vec!["include".into(), "src".into()] }

/// `glfw = "*"` / `glfw = "3.3.9"`  ou  `minhalib = { git = "...", tag = "...", ... }`
#[derive(Deserialize)]
#[serde(untagged)]
enum DepSpec {
    Version(String),
    Custom(CustomDep),
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct CustomDep {
    git: String,
    tag: String,
    kind: Option<String>, // cmake | header | sources | system
    subdir: String,
    cmake: Vec<String>,
    libs: Vec<String>,
    libs_win: Vec<String>,
    sys: Vec<String>,
    sys_win: Vec<String>,
    inc: Vec<String>,
    defines: Vec<String>,
    deps: Vec<String>,
    sources: Vec<String>,
    shared: bool,
    sdk_env: String,
    sdk_dir: String,
    libdir: String,
    libdir_win: String,
}

// ───────────────────────────── CLI ─────────────────────────────

#[derive(Parser)]
#[command(name = "rsfmake", version, about = "Gerenciador de pacotes/build C++ sobre CMake + Ninja")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Cria um novo projeto
    New { name: String },
    /// Cria rsfmake.toml no diretório atual
    Init,
    /// Compila o projeto (e as dependências)
    Build(BuildArgs),
    /// Compila e executa
    Run {
        #[command(flatten)]
        b: BuildArgs,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Só baixa e compila as dependências
    Deps(BuildArgs),
    /// Remove bin/ (--all remove também redist/)
    Clean {
        #[arg(long)]
        all: bool,
    },
    /// Lista as bibliotecas embutidas
    List,
}

#[derive(Args, Clone)]
struct BuildArgs {
    /// Build release (padrão: debug)
    #[arg(short, long)]
    release: bool,
    /// linux | windows (padrão: o host)
    #[arg(long)]
    target: Option<String>,
    /// x64 | arm64 (padrão: a do host)
    #[arg(long)]
    arch: Option<String>,
}

// ───────────────────────────── Contexto ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Plat { Linux, Windows }
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arch { X64, Arm64 }

impl Plat {
    fn s(self) -> &'static str { match self { Plat::Linux => "linux", Plat::Windows => "windows" } }
}
impl Arch {
    fn s(self) -> &'static str { match self { Arch::X64 => "x64", Arch::Arm64 => "arm64" } }
}

struct Ctx {
    root: PathBuf,
    target: Plat,
    arch: Arch,
    release: bool,
    env: Vec<(String, String)>,
    toolchain: Option<PathBuf>,
}

impl Ctx {
    fn new(root: PathBuf, a: &BuildArgs) -> Result<Self> {
        let host = if cfg!(windows) { Plat::Windows } else { Plat::Linux };
        let host_arch = if cfg!(target_arch = "aarch64") { Arch::Arm64 } else { Arch::X64 };
        let target = match a.target.as_deref() {
            None => host,
            Some("linux") => Plat::Linux,
            Some("windows") | Some("win") | Some("win32") => Plat::Windows,
            Some(o) => bail!("target inválido '{o}' (use linux ou windows)"),
        };
        let arch = match a.arch.as_deref() {
            None => host_arch,
            Some("x64") | Some("x86_64") | Some("amd64") => Arch::X64,
            Some("arm64") | Some("aarch64") => Arch::Arm64,
            Some(o) => bail!("arch inválida '{o}' (use x64 ou arm64)"),
        };
        Ok(Ctx { root, target, arch, release: a.release, env: vec![], toolchain: None })
    }

    fn cfg(&self) -> &'static str { if self.release { "release" } else { "debug" } }

    /// "linux", "windows", ou "linux_arm64" quando não é x64
    fn platdir(&self) -> String {
        if self.arch == Arch::X64 { self.target.s().into() } else { format!("{}_{}", self.target.s(), self.arch.s()) }
    }
    /// redist/<plataforma>/<arch>
    fn prefix(&self) -> PathBuf { self.root.join("redist").join(self.target.s()).join(self.arch.s()) }
    /// bin/<plataforma>+<config>
    fn out(&self) -> PathBuf { self.root.join("bin").join(format!("{}+{}", self.platdir(), self.cfg())) }
    /// bin/build_temp
    fn tmp(&self) -> PathBuf { self.root.join("bin").join("build_temp") }

    fn cmd(&self, prog: &str) -> Command {
        let mut c = Command::new(prog);
        for (k, v) in &self.env { c.env(k, v); }
        c
    }

    /// Escolhe/configura o toolchain conforme host → alvo.
    fn setup_toolchain(&mut self) -> Result<()> {
        let host_win = cfg!(windows);
        let host_arch = if cfg!(target_arch = "aarch64") { Arch::Arm64 } else { Arch::X64 };
        std::fs::create_dir_all(self.tmp())?;

        for t in ["cmake", "git"] {
            if which(t).is_none() { bail!("'{t}' não encontrado no PATH"); }
        }
        if !host_win && which("ninja").is_none() {
            bail!("'ninja' não encontrado (sudo apt install ninja-build)");
        }

        match (host_win, self.target) {
            (true, Plat::Linux) => bail!("compilar para Linux a partir do Windows não é suportado (use WSL)"),
            (true, Plat::Windows) => {
                if which("cl").is_none() {
                    eprintln!("[rsfmake] carregando ambiente do MSVC...");
                    self.env = msvc_env(self.arch)?;
                }
            }
            (false, Plat::Windows) => {
                if self.arch != Arch::X64 { bail!("cross para Windows ARM64 não é suportado a partir do Linux"); }
                let pre = "x86_64-w64-mingw32";
                let pick = |b: &str| {
                    let posix = format!("{pre}-{b}-posix");
                    if which(&posix).is_some() { posix } else { format!("{pre}-{b}") }
                };
                let (cc, cxx) = (pick("gcc"), pick("g++"));
                if which(&cxx).is_none() { bail!("MinGW não encontrado (sudo apt install g++-mingw-w64-x86-64)"); }
                let tc = format!(
                    "set(CMAKE_SYSTEM_NAME Windows)\nset(CMAKE_SYSTEM_PROCESSOR x86_64)\n\
                     set(CMAKE_C_COMPILER {cc})\nset(CMAKE_CXX_COMPILER {cxx})\n\
                     set(CMAKE_RC_COMPILER {pre}-windres)\n\
                     set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)\n"
                );
                self.write_toolchain(&tc)?;
            }
            (false, Plat::Linux) => {
                if self.arch != host_arch {
                    let (triple, proc) = match self.arch {
                        Arch::Arm64 => ("aarch64-linux-gnu", "aarch64"),
                        Arch::X64 => ("x86_64-linux-gnu", "x86_64"),
                    };
                    if which(&format!("{triple}-g++")).is_none() {
                        bail!("compilador cruzado '{triple}-g++' não encontrado (sudo apt install g++-{triple})");
                    }
                    let tc = format!(
                        "set(CMAKE_SYSTEM_NAME Linux)\nset(CMAKE_SYSTEM_PROCESSOR {proc})\n\
                         set(CMAKE_C_COMPILER {triple}-gcc)\nset(CMAKE_CXX_COMPILER {triple}-g++)\n\
                         set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)\n"
                    );
                    self.write_toolchain(&tc)?;
                }
            }
        }
        Ok(())
    }

    fn write_toolchain(&mut self, content: &str) -> Result<()> {
        let p = self.tmp().join(format!("toolchain-{}.cmake", self.platdir()));
        std::fs::write(&p, content)?;
        self.toolchain = Some(p);
        Ok(())
    }
}

// ───────────────────────────── Utilitários ─────────────────────────────

fn fwd(p: &Path) -> String { p.to_string_lossy().replace('\\', "/") }

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exts: &[&str] = if cfg!(windows) { &[".exe", ".cmd", ".bat", ""] } else { &[""] };
    for dir in std::env::split_paths(&path) {
        for e in exts {
            let p = dir.join(format!("{name}{e}"));
            if p.is_file() { return Some(p); }
        }
    }
    None
}

fn run(cmd: &mut Command, what: &str) -> Result<()> {
    let st = cmd.status().with_context(|| format!("não consegui executar {what} (está no PATH?)"))?;
    if !st.success() { bail!("{what} falhou ({st})"); }
    Ok(())
}

fn dedup(v: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    v.into_iter().filter(|x| seen.insert(x.clone())).collect()
}

fn quote(v: &[String]) -> String {
    v.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(" ")
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for e in std::fs::read_dir(src)? {
        let e = e?;
        let to = dst.join(e.file_name());
        if e.file_type()?.is_dir() { copy_dir(&e.path(), &to)?; } else { std::fs::copy(e.path(), to)?; }
    }
    Ok(())
}

#[cfg(windows)]
fn msvc_env(arch: Arch) -> Result<Vec<(String, String)>> {
    use std::os::windows::process::CommandExt;
    let pf = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    let vswhere = PathBuf::from(pf).join(r"Microsoft Visual Studio\Installer\vswhere.exe");
    let out = Command::new(&vswhere)
        .args(["-latest", "-products", "*", "-requires",
               "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"])
        .output()
        .context("vswhere não encontrado: instale o Visual Studio / Build Tools com o workload C++")?;
    let inst = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if inst.is_empty() { bail!("Visual Studio com ferramentas C++ não encontrado"); }
    let bat = PathBuf::from(inst).join(r"VC\Auxiliary\Build\vcvarsall.bat");
    let a = match arch { Arch::X64 => "x64", Arch::Arm64 => "x64_arm64" };
    let out = Command::new("cmd")
        .raw_arg(format!("/C \"\"{}\" {} >nul && set\"", bat.display(), a))
        .output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let vars: Vec<(String, String)> = text
        .lines()
        .filter_map(|l| l.split_once('='))
        .filter(|(k, _)| !k.is_empty() && !k.starts_with('='))
        .map(|(k, v)| (k.to_string(), v.trim_end().to_string()))
        .collect();
    if vars.is_empty() { bail!("falha ao carregar o ambiente do MSVC"); }
    Ok(vars)
}

#[cfg(not(windows))]
fn msvc_env(_: Arch) -> Result<Vec<(String, String)>> { bail!("MSVC só existe no Windows") }

// ───────────────────────────── Dependências ─────────────────────────────

fn custom_to_lib(name: &str, c: &CustomDep) -> Result<Lib> {
    let kind = match c.kind.as_deref() {
        Some("cmake") => Kind::Cmake,
        Some("header") => Kind::Header,
        Some("sources") => Kind::Sources,
        Some("system") => Kind::System,
        Some("prebuilt") => Kind::Prebuilt,
        Some(o) => bail!("kind inválido '{o}' em [{name}]"),
        None if !c.libs.is_empty() || !c.cmake.is_empty() => Kind::Cmake,
        None if !c.sources.is_empty() => Kind::Sources,
        None => Kind::Header,
    };
    if !matches!(kind, Kind::System | Kind::Prebuilt) && c.git.is_empty() { bail!("dependência '{name}' sem 'git'"); }
    let mut l = Lib::new(name, &c.git, if c.tag.is_empty() { "master" } else { &c.tag }, kind);
    l.subdir = c.subdir.clone();
    l.opts = c.cmake.clone();
    l.libs = c.libs.clone();
    l.libs_win = c.libs_win.clone();
    l.sys_linux = c.sys.clone();
    l.sys_win = c.sys_win.clone();
    l.inc = c.inc.clone();
    l.defs = c.defines.clone();
    l.deps = c.deps.clone();
    l.srcs = c.sources.clone();
    l.shared = c.shared;
    l.sdk_env = c.sdk_env.clone();
    l.sdk_dir = if c.sdk_dir.is_empty() { name.to_string() } else { c.sdk_dir.clone() };
    l.libdir_linux = c.libdir.clone();
    l.libdir_win = if c.libdir_win.is_empty() { c.libdir.clone() } else { c.libdir_win.clone() };
    l.help = format!("Extraia o SDK de '{name}' em redist/_sdk/{} (ou defina {})", l.sdk_dir, l.sdk_env);
    Ok(l)
}

/// No Windows não existe ncurses nativo: usa PDCursesMod (que expõe a mesma API).
fn curses_alias(name: &str, win: bool) -> &str {
    if win && matches!(name, "ncurses" | "ncursesw") { "pdcurses" } else { name }
}

/// Expande dependências transitivas; resultado em ordem "deps primeiro".
fn resolve(m: &Manifest, win: bool) -> Result<Vec<Lib>> {
    fn visit(
        orig: &str,
        win: bool,
        wanted: &BTreeMap<String, DepSpec>,
        reg: &HashMap<String, Lib>,
        out: &mut Vec<Lib>,
        seen: &mut HashSet<String>,
    ) -> Result<()> {
        let name = curses_alias(orig, win);
        if !win && name == "pdcurses" {
            bail!("'pdcurses' só é suportado no Windows; no Linux use 'ncurses' (ou 'ncursesw')");
        }
        if !seen.insert(name.to_string()) { return Ok(()); }
        let swapped = name != orig;
        let lib = match wanted.get(orig) {
            Some(DepSpec::Custom(c)) if !swapped => custom_to_lib(name, c)?,
            other => {
                let mut l = reg.get(name).cloned()
                    .ok_or_else(|| anyhow!("biblioteca desconhecida '{name}' (veja `rsfmake list` ou use a forma {{ git = ... }})"))?;
                if let (false, Some(DepSpec::Version(v))) = (swapped, other) {
                    if !v.is_empty() && v != "*" { l.tag = v.clone(); }
                }
                l
            }
        };
        for d in lib.deps.clone() { visit(&d, win, wanted, reg, out, seen)?; }
        out.push(lib);
        Ok(())
    }
    let reg = registry();
    let (mut out, mut seen) = (vec![], HashSet::new());
    for name in m.dependencies.keys() { visit(name, win, &m.dependencies, &reg, &mut out, &mut seen)?; }
    Ok(out)
}

fn fetch(ctx: &Ctx, lib: &Lib) -> Result<PathBuf> {
    let base = ctx.root.join("redist").join("_src");
    let dir = base.join(lib.repo_dir());
    if dir.exists() { return Ok(dir); }
    std::fs::create_dir_all(&base)?;
    let part = base.join(format!("{}.partial", lib.repo_dir()));
    if part.exists() { std::fs::remove_dir_all(&part)?; }
    eprintln!("[rsfmake] baixando {} @ {}", lib.name, lib.tag);
    run(
        Command::new("git")
            .args(["clone", "--depth", "1", "--recurse-submodules", "--shallow-submodules", "--branch"])
            .arg(&lib.tag).arg(&lib.git).arg(&part),
        "git clone",
    )?;
    std::fs::rename(&part, &dir)?;
    Ok(dir)
}

fn build_cmake_dep(ctx: &Ctx, l: &Lib, src: &Path) -> Result<()> {
    let prefix = ctx.prefix();
    let stamp = prefix.join(".stamps").join(format!("{}-{}.ok", l.name, l.tag.replace('/', "_")));
    if stamp.exists() {
        eprintln!("[rsfmake] {} (em cache)", l.name);
        return Ok(());
    }
    let bdir = ctx.tmp().join("deps").join(ctx.platdir()).join(&l.name);
    let sdir = if l.subdir.is_empty() { src.to_path_buf() } else { src.join(&l.subdir) };
    eprintln!("[rsfmake] compilando {} -> {}", l.name, prefix.display());

    let mut c = ctx.cmd("cmake");
    c.arg("-S").arg(&sdir).arg("-B").arg(&bdir).args([
        "-G", "Ninja",
        "-DCMAKE_BUILD_TYPE=Release",
        "-DCMAKE_INSTALL_LIBDIR=lib",
        "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        "-DCMAKE_POLICY_VERSION_MINIMUM=3.5",
        "-DCMAKE_POLICY_DEFAULT_CMP0091=NEW",
        "-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL",
    ]);
    c.arg(format!("-DCMAKE_INSTALL_PREFIX={}", fwd(&prefix)));
    c.arg(format!("-DCMAKE_PREFIX_PATH={}", fwd(&prefix)));
    c.arg(format!("-DBUILD_SHARED_LIBS={}", if l.shared { "ON" } else { "OFF" }));
    // UTF-8: MSVC nativo no Windows; GCC/Clang/MinGW nos demais casos
    if cfg!(windows) && ctx.target == Plat::Windows {
        c.arg("-DCMAKE_C_FLAGS=/utf-8");
        c.arg("-DCMAKE_CXX_FLAGS=/utf-8");
    } else {
        c.arg("-DCMAKE_C_FLAGS=-finput-charset=UTF-8 -fexec-charset=UTF-8");
        c.arg("-DCMAKE_CXX_FLAGS=-finput-charset=UTF-8 -fexec-charset=UTF-8");
    }
    if let Some(t) = &ctx.toolchain { c.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", fwd(t))); }
    c.args(&l.opts);
    run(&mut c, &format!("cmake configure ({})", l.name))?;
    run(ctx.cmd("cmake").arg("--build").arg(&bdir), &format!("build ({})", l.name))?;
    run(ctx.cmd("cmake").arg("--install").arg(&bdir), &format!("install ({})", l.name))?;

    std::fs::create_dir_all(stamp.parent().unwrap())?;
    std::fs::write(stamp, "ok")?;
    Ok(())
}

/// SDK proprietário baixado à mão: copia binários para redist/<plat>/<arch>/{lib,bin}.
fn install_prebuilt(ctx: &Ctx, l: &Lib) -> Result<PathBuf> {
    let root = match std::env::var(&l.sdk_env) {
        Ok(p) if !l.sdk_env.is_empty() => PathBuf::from(p),
        _ => ctx.root.join("redist").join("_sdk").join(&l.sdk_dir),
    };
    if !root.is_dir() {
        bail!("SDK de '{}' não encontrado em {}.\n{}", l.name, root.display(), l.help);
    }
    let win = ctx.target == Plat::Windows;
    let token = match (win, ctx.arch) {
        (true, Arch::X64) => "x64",
        (false, Arch::X64) => "x86_64",
        (_, Arch::Arm64) => "arm64",
    };
    let tpl = if win { &l.libdir_win } else { &l.libdir_linux };
    let dir = root.join(tpl.replace("{arch}", token));
    if !dir.is_dir() {
        bail!("pasta de bibliotecas não encontrada: {}\n{}", dir.display(), l.help);
    }
    let (lib_out, bin_out) = (ctx.prefix().join("lib"), ctx.prefix().join("bin"));
    std::fs::create_dir_all(&lib_out)?;
    std::fs::create_dir_all(&bin_out)?;
    for e in std::fs::read_dir(&dir)?.flatten() {
        let p = e.path();
        if !p.is_file() { continue; }
        let n = e.file_name().to_string_lossy().to_string();
        if win {
            if n.ends_with(".dll") { std::fs::copy(&p, bin_out.join(&n))?; }
            else if n.ends_with(".lib") { std::fs::copy(&p, lib_out.join(&n))?; }
        } else if n.contains(".so") {
            std::fs::copy(&p, lib_out.join(&n))?;
        }
    }
    eprintln!("[rsfmake] {} instalado de {}", l.name, root.display());
    Ok(root)
}

/// Cria <prefixo>/include/ncurses.h -> <curses.h>, para o mesmo `#include <ncurses.h>` valer no Linux e no Windows.
fn write_ncurses_shim(ctx: &Ctx) -> Result<()> {
    let dir = ctx.prefix().join("include");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("ncurses.h");
    let body = "#pragma once\n// gerado pelo rsfmake: ncurses no Windows = PDCursesMod\n#include <curses.h>\n";
    if std::fs::read_to_string(&path).map(|s| s != body).unwrap_or(true) {
        std::fs::write(&path, body)?;
    }
    Ok(())
}

fn prepare_deps(ctx: &Ctx, libs: &[Lib]) -> Result<Vec<(Lib, Option<PathBuf>)>> {
    let mut out = vec![];
    for l in libs {
        if l.kind == Kind::System { out.push((l.clone(), None)); continue; }
        if l.kind == Kind::Prebuilt {
            let sdk = install_prebuilt(ctx, l)?;
            out.push((l.clone(), Some(sdk)));
            continue;
        }
        let src = fetch(ctx, l)?;
        if l.kind == Kind::Cmake { build_cmake_dep(ctx, l, &src)?; }
        if l.name == "pdcurses" { write_ncurses_shim(ctx)?; }
        out.push((l.clone(), Some(src)));
    }
    Ok(out)
}

fn include_dirs(ctx: &Ctx, l: &Lib, src: Option<&Path>) -> Vec<String> {
    let base = match l.kind {
        // libs do sistema: `inc` são caminhos absolutos (ex: /usr/include/mysql)
        Kind::System => return l.inc.iter().map(|i| i.replace('\\', "/")).collect(),
        Kind::Cmake => ctx.prefix(),
        _ => match src { Some(s) => s.to_path_buf(), None => return vec![] },
    };
    l.inc.iter().map(|i| fwd(&base.join(i))).collect()
}

/// (includes, link dirs) extras de libs do sistema, ex: Vulkan SDK.
fn system_paths(ctx: &Ctx, l: &Lib) -> (Vec<String>, Vec<String>) {
    if l.name == "vulkan" {
        if let Ok(sdk) = std::env::var("VULKAN_SDK") {
            let sdk = PathBuf::from(sdk);
            return if ctx.target == Plat::Windows {
                (vec![fwd(&sdk.join("Include"))], vec![fwd(&sdk.join("Lib"))])
            } else {
                (vec![fwd(&sdk.join("include"))], vec![fwd(&sdk.join("lib"))])
            };
        }
    }
    (vec![], vec![])
}

// ───────────────────────────── Geração do CMakeLists ─────────────────────────────

fn gen_cmake(ctx: &Ctx, m: &Manifest, deps: &[(Lib, Option<PathBuf>)]) -> Result<String> {
    let p = &m.package;
    let name = &p.name;
    let win = ctx.target == Plat::Windows;
    let mut c = String::new();

    writeln!(c, "cmake_minimum_required(VERSION 3.20)")?;
    writeln!(c, "set(CMAKE_MSVC_RUNTIME_LIBRARY \"MultiThreadedDLL\")")?;
    writeln!(c, "project({name} LANGUAGES C CXX)")?;
    writeln!(c, "set(CMAKE_CXX_STANDARD {})", p.std)?;
    writeln!(c, "set(CMAKE_CXX_STANDARD_REQUIRED ON)\nset(CMAKE_CXX_EXTENSIONS OFF)\nset(CMAKE_C_STANDARD 11)")?;
    let out = fwd(&ctx.out());
    for v in ["RUNTIME", "LIBRARY", "ARCHIVE"] {
        writeln!(c, "set(CMAKE_{v}_OUTPUT_DIRECTORY \"{out}\")")?;
    }

    // fontes do projeto
    let root = fwd(&ctx.root);
    let pats: Vec<String> = p.sources.iter().map(|s| format!("{root}/{s}")).collect();
    writeln!(c, "file(GLOB_RECURSE RSF_SOURCES CONFIGURE_DEPENDS {})", quote(&pats))?;

    // fontes de deps do tipo Sources (imgui, ...)
    let (mut dep_src, mut dep_glob) = (vec![], vec![]);
    for (l, src) in deps {
        if l.kind == Kind::Sources {
            if let Some(s) = src {
                for f in &l.srcs {
                    let full = fwd(&s.join(f));
                    if f.contains('*') { dep_glob.push(full); } else { dep_src.push(full); }
                }
            }
        }
    }
    writeln!(c, "set(RSF_DEP_SOURCES {})", quote(&dep_src))?;
    if !dep_glob.is_empty() {
        writeln!(c, "file(GLOB RSF_DEP_GLOB {})", quote(&dep_glob))?;
        writeln!(c, "list(APPEND RSF_DEP_SOURCES ${{RSF_DEP_GLOB}})")?;
    }

    match p.kind.as_str() {
        "exe" => writeln!(c, "add_executable({name} ${{RSF_SOURCES}} ${{RSF_DEP_SOURCES}})")?,
        "static" => writeln!(c, "add_library({name} STATIC ${{RSF_SOURCES}} ${{RSF_DEP_SOURCES}})")?,
        "shared" => writeln!(c, "add_library({name} SHARED ${{RSF_SOURCES}} ${{RSF_DEP_SOURCES}})")?,
        k => bail!("type inválido '{k}' (use exe, static ou shared)"),
    }

    // includes / defines / link
    let own_inc: Vec<String> = p.include.iter().map(|i| format!("{root}/{i}")).collect();
    let mut sys_inc = vec![fwd(&ctx.prefix().join("include"))];
    let mut link_dirs = vec![fwd(&ctx.prefix().join("lib"))];
    let mut defs = p.defines.clone();
    for (l, src) in deps {
        sys_inc.extend(include_dirs(ctx, l, src.as_deref()));
        defs.extend(l.defs.clone());
        let (i, d) = system_paths(ctx, l);
        sys_inc.extend(i);
        link_dirs.extend(d);
    }
    // ordem de link: dependentes antes das dependências (importa para .a no Linux)
    let mut link = vec![];
    let mut sys = vec![];
    for (l, _) in deps.iter().rev() {
        link.extend(l.link_names(win).iter().cloned());
        sys.extend(l.sys_for(win).iter().cloned());
    }
    let mut all_link = dedup(link);
    all_link.extend(dedup(sys));
    all_link.extend(p.libs.clone());

    writeln!(c, "target_include_directories({name} PRIVATE {})", quote(&own_inc))?;
    writeln!(c, "target_include_directories({name} SYSTEM PRIVATE {})", quote(&dedup(sys_inc)))?;
    if !defs.is_empty() {
        writeln!(c, "target_compile_definitions({name} PRIVATE {})", dedup(defs).join(" "))?;
    }
    // UTF-8 em fontes e strings literais (MSVC e GCC/Clang/MinGW)
    writeln!(c, "if(MSVC)")?;
    writeln!(c, "  target_compile_options({name} PRIVATE /utf-8)")?;
    writeln!(c, "else()")?;
    writeln!(c, "  target_compile_options({name} PRIVATE -finput-charset=UTF-8 -fexec-charset=UTF-8)")?;
    writeln!(c, "endif()")?;
    if !p.cxxflags.is_empty() {
        writeln!(c, "target_compile_options({name} PRIVATE {})", quote(&p.cxxflags))?;
    }
    writeln!(c, "target_link_directories({name} PRIVATE {})", quote(&dedup(link_dirs)))?;
    if !all_link.is_empty() {
        writeln!(c, "target_link_libraries({name} PRIVATE {})", all_link.join(" "))?;
    }

    if !win {
        writeln!(c, "set_target_properties({name} PROPERTIES BUILD_RPATH \"\\$ORIGIN\")")?;
    }
    if win && !cfg!(windows) {
        // MinGW cross: runtime estático para não depender de DLLs do gcc
        writeln!(c, "target_link_options({name} PRIVATE -static)")?;
    }
    Ok(c)
}

// ───────────────────────────── Comandos ─────────────────────────────

fn find_root() -> Result<PathBuf> {
    let mut d = std::env::current_dir()?;
    loop {
        if d.join("rsfmake.toml").is_file() { return Ok(d); }
        if !d.pop() { bail!("rsfmake.toml não encontrado (use `rsfmake new` ou `rsfmake init`)"); }
    }
}

fn load_manifest(root: &Path) -> Result<Manifest> {
    let txt = std::fs::read_to_string(root.join("rsfmake.toml"))?;
    toml::from_str(&txt).context("rsfmake.toml inválido")
}

fn copy_runtime(ctx: &Ctx, out: &Path) -> Result<()> {
    let prefix = ctx.prefix();
    if ctx.target == Plat::Windows {
        if let Ok(rd) = std::fs::read_dir(prefix.join("bin")) {
            for e in rd.flatten() {
                if e.path().extension().is_some_and(|x| x == "dll") {
                    std::fs::copy(e.path(), out.join(e.file_name()))?;
                }
            }
        }
    } else if let Ok(rd) = std::fs::read_dir(prefix.join("lib")) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().contains(".so") && e.path().is_file() {
                std::fs::copy(e.path(), out.join(e.file_name()))?;
            }
        }
    }
    Ok(())
}

fn cmd_build(root: &Path, b: &BuildArgs, deps_only: bool) -> Result<Option<PathBuf>> {
    let m = load_manifest(root)?;
    let mut ctx = Ctx::new(root.to_path_buf(), b)?;
    ctx.setup_toolchain()?;

    let libs = resolve(&m, ctx.target == Plat::Windows)?;
    let deps = prepare_deps(&ctx, &libs)?;
    if deps_only { return Ok(None); }

    let name = format!("{}+{}", ctx.platdir(), ctx.cfg());
    let gen = ctx.tmp().join(&name).join("gen");
    let bld = ctx.tmp().join(&name).join("build");
    std::fs::create_dir_all(&gen)?;
    std::fs::create_dir_all(ctx.out())?;
    std::fs::write(gen.join("CMakeLists.txt"), gen_cmake(&ctx, &m, &deps)?)?;

    let mut c = ctx.cmd("cmake");
    c.arg("-S").arg(&gen).arg("-B").arg(&bld).args(["-G", "Ninja", "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON"]);
    c.arg(format!("-DCMAKE_BUILD_TYPE={}", if ctx.release { "Release" } else { "Debug" }));
    if let Some(t) = &ctx.toolchain { c.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", fwd(t))); }
    run(&mut c, "cmake configure")?;
    run(ctx.cmd("cmake").arg("--build").arg(&bld), "cmake build")?;

    // compile_commands.json na raiz (clangd / KDevelop / VSCode)
    let _ = std::fs::copy(bld.join("compile_commands.json"), root.join("compile_commands.json"));

    let out = ctx.out();
    copy_runtime(&ctx, &out)?;
    if let Some(a) = &m.package.assets {
        let src = root.join(a);
        if src.is_dir() { copy_dir(&src, &out.join(a))?; }
    }

    let exe = out.join(if ctx.target == Plat::Windows { format!("{}.exe", m.package.name) } else { m.package.name.clone() });
    eprintln!("[rsfmake] pronto: {}", out.display());
    Ok(Some(exe))
}

fn scaffold(dir: &Path, name: &str) -> Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    let toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
type = "exe"        # exe | static | shared
std = 20
# sources = ["src/*.cpp"]
# include = ["include", "src"]
# defines = []
# assets  = "assets"

[dependencies]
nlohmann_json = "*"
# glfw = "*"
# opengl = "*"
# vulkan = "*"
# glm = "*"
# tinygltf = "*"
# bullet3 = "*"
# imgui_glfw_gl3 = "*"
# ncurses = "*"     # TUI: ncurses no Linux, PDCursesMod no Windows (#include <ncurses.h> nos dois)

# Lib customizada (qualquer repo git com CMake):
# fmod = "*"        # SDK proprietário: extraia em redist/_sdk/fmod
# minhalib = {{ git = "https://github.com/x/y.git", tag = "v1.0", cmake = ["-DY_TESTS=OFF"], libs = ["y"], inc = ["include/y"] }}
"#
    );
    let main = r##"#include <nlohmann/json.hpp>
#include <iostream>

int main() {
    nlohmann::json j = {{"ferramenta", "rsfmake"}, {"ok", true}};
    std::cout << j.dump(2) << "\n";
    return 0;
}
"##;
    let write_new = |p: PathBuf, s: &str| -> Result<()> { if !p.exists() { std::fs::write(p, s)?; } Ok(()) };
    write_new(dir.join("rsfmake.toml"), &toml)?;
    write_new(dir.join("src/main.cpp"), main)?;
    write_new(dir.join(".gitignore"), "bin/\nredist/\ncompile_commands.json\n")?;
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::New { name } => {
            let dir = PathBuf::from(&name);
            if dir.exists() { bail!("'{name}' já existe"); }
            scaffold(&dir, &name)?;
            eprintln!("[rsfmake] projeto '{name}' criado. cd {name} && rsfmake run");
        }
        Cmd::Init => {
            let cwd = std::env::current_dir()?;
            let name = cwd.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "app".into());
            scaffold(&cwd, &name)?;
        }
        Cmd::Build(b) => { cmd_build(&find_root()?, &b, false)?; }
        Cmd::Deps(b) => { cmd_build(&find_root()?, &b, true)?; }
        Cmd::Run { b, args } => {
            let host_win = cfg!(windows);
            if let Some(t) = b.target.as_deref() {
                if (t == "windows") != host_win { bail!("não dá pra executar um binário de outra plataforma"); }
            }
            let exe = cmd_build(&find_root()?, &b, false)?.unwrap();
            let st = Command::new(&exe).args(args).current_dir(exe.parent().unwrap()).status()?;
            std::process::exit(st.code().unwrap_or(1));
        }
        Cmd::Clean { all } => {
            let root = find_root()?;
            let _ = std::fs::remove_dir_all(root.join("bin"));
            if all { let _ = std::fs::remove_dir_all(root.join("redist")); }
            eprintln!("[rsfmake] limpo");
        }
        Cmd::List => {
            let reg = registry();
            let mut v: Vec<_> = reg.values().collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            for l in v {
                let k = match l.kind { Kind::Cmake => "cmake", Kind::Header => "header", Kind::Sources => "sources", Kind::System => "system", Kind::Prebuilt => "sdk" };
                println!("{:<18} {:<8} {}", l.name, k, l.tag);
            }
        }
    }
    Ok(())
}