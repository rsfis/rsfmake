//! Registro embutido de bibliotecas.
//!
//! Tipos de lib (Kind):
//!   Cmake   -> clona, compila com CMake+Ninja e instala em redist/<plat>/<arch>
//!              (`inc` é relativo ao prefixo de instalação)
//!   Header  -> só clona; `inc` é relativo ao diretório do código-fonte
//!   Sources -> clona e compila os `srcs` junto com o projeto (ex: imgui)
//!   System  -> lib do sistema (opengl, vulkan), nada é baixado
//!   Sources -> aceita globs em `srcs` (ex: "pdcurses/*.c")
//!   Prebuilt-> SDK proprietário baixado À MÃO (fmod, steamworks). Procurado na
//!              variável de ambiente `sdk` ou em redist/_sdk/<sdk_dir>; os binários
//!              são copiados para redist/<plat>/<arch>/{lib,bin}.
//!              `{arch}` em libdir vira x86_64/arm64 (linux) ou x64/arm64 (windows)

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
      #[default]
      Header,
      Cmake,
      Sources,
      System,
      Prebuilt,
}

#[derive(Clone, Debug, Default)]
pub struct Lib {
      pub name: String,
      pub git: String,
      pub tag: String,
      pub kind: Kind,
      pub subdir: String,
      pub opts: Vec<String>,
      pub libs: Vec<String>,
      pub libs_win: Vec<String>,
      pub sys_linux: Vec<String>,
      pub sys_win: Vec<String>,
      pub inc: Vec<String>,
      pub defs: Vec<String>,
      pub deps: Vec<String>,
      pub srcs: Vec<String>,
      pub shared: bool,
      pub sdk_env: String,
      pub sdk_dir: String,
      pub libdir_linux: String,
      pub libdir_win: String,
      pub help: String,
}

fn v(x: &[&str]) -> Vec<String> {
      x.iter().map(|s| s.to_string()).collect()
}

impl Lib {
      pub fn new(name: &str, git: &str, tag: &str, kind: Kind) -> Self {
            Lib { name: name.into(), git: git.into(), tag: tag.into(), kind, ..Default::default() }
      }
      fn opts(mut self, x: &[&str]) -> Self { self.opts = v(x); self }
      fn libs(mut self, x: &[&str]) -> Self { self.libs = v(x); self }
      fn libs_win(mut self, x: &[&str]) -> Self { self.libs_win = v(x); self }
      fn sys_linux(mut self, x: &[&str]) -> Self { self.sys_linux = v(x); self }
      fn sys_win(mut self, x: &[&str]) -> Self { self.sys_win = v(x); self }
      fn inc(mut self, x: &[&str]) -> Self { self.inc = v(x); self }
      fn defs(mut self, x: &[&str]) -> Self { self.defs = v(x); self }
      fn deps(mut self, x: &[&str]) -> Self { self.deps = v(x); self }
      fn srcs(mut self, x: &[&str]) -> Self { self.srcs = v(x); self }
      fn subdir(mut self, x: &str) -> Self { self.subdir = x.into(); self }
      fn shared(mut self) -> Self { self.shared = true; self }
      fn prebuilt(mut self, env: &str, dir: &str, libdir_linux: &str, libdir_win: &str, help: &str) -> Self {
            self.sdk_env = env.into();
            self.sdk_dir = dir.into();
            self.libdir_linux = libdir_linux.into();
            self.libdir_win = libdir_win.into();
            self.help = help.into();
            self
      }

      /// Nomes de link para a plataforma alvo.
      pub fn link_names(&self, win: bool) -> &[String] {
            if win && !self.libs_win.is_empty() { &self.libs_win } else { &self.libs }
      }

      pub fn sys_for(&self, win: bool) -> &[String] {
            if win { &self.sys_win } else { &self.sys_linux }
      }

      /// Nome do diretório de fonte (compartilhado entre libs do mesmo repo+tag).
      pub fn repo_dir(&self) -> String {
            let base = self.git.trim_end_matches('/').rsplit('/').next().unwrap_or("repo");
            let base = base.trim_end_matches(".git");
            format!("{}-{}", base, self.tag.replace('/', "_"))
      }
}

pub fn registry() -> HashMap<String, Lib> {
      use Kind::*;
      let gh = |p: &str| format!("https://github.com/{p}.git");

      let list = vec![
            // ---------- Sistema ----------
            Lib::new("opengl", "", "", System).libs(&["GL"]).libs_win(&["opengl32"]),
            Lib::new("vulkan", "", "", System).libs(&["vulkan"]).libs_win(&["vulkan-1"]),

            // ---------- Janela / input ----------
            Lib::new("glfw", &gh("glfw/glfw"), "3.4", Cmake)
            .opts(&["-DGLFW_BUILD_EXAMPLES=OFF", "-DGLFW_BUILD_TESTS=OFF",
                  "-DGLFW_BUILD_DOCS=OFF", "-DGLFW_INSTALL=ON"])
            .libs(&["glfw3"])
            .sys_linux(&["pthread", "dl", "m"])
            .sys_win(&["gdi32", "user32", "shell32"]),
            Lib::new("sdl2", &gh("libsdl-org/SDL"), "release-2.30.9", Cmake).shared()
            .opts(&["-DSDL_SHARED=ON", "-DSDL_STATIC=OFF", "-DSDL_TEST=OFF"])
            .libs(&["SDL2"]).libs_win(&["SDL2main", "SDL2"])
            .inc(&["include/SDL2"]),
            Lib::new("sdl3", &gh("libsdl-org/SDL"), "release-3.2.0", Cmake).shared()
            .opts(&["-DSDL_SHARED=ON", "-DSDL_STATIC=OFF", "-DSDL_TESTS=OFF",
                  "-DSDL_EXAMPLES=OFF"])
            .libs(&["SDL3"]),

            // ---------- Vulkan ----------
            Lib::new("vulkan_headers", &gh("KhronosGroup/Vulkan-Headers"), "v1.3.296", Header)
            .inc(&["include"]),
            Lib::new("vma", &gh("GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator"), "v3.1.0", Header)
            .inc(&["include"]).deps(&["vulkan_headers"]),

            // ---------- Dados / formatos ----------
            Lib::new("nlohmann_json", &gh("nlohmann/json"), "v3.11.3", Header).inc(&["include"]),
            Lib::new("tinygltf", &gh("syoyo/tinygltf"), "v2.9.3", Header).inc(&["."]),
            Lib::new("cgltf", &gh("jkuhlmann/cgltf"), "v1.14", Header).inc(&["."]),
            Lib::new("tinyobjloader", &gh("tinyobjloader/tinyobjloader"), "v2.0.0rc13", Header)
            .inc(&["."]),
            Lib::new("stb", &gh("nothings/stb"), "master", Header).inc(&["."]),
            Lib::new("rapidjson", &gh("Tencent/rapidjson"), "v1.1.0", Header).inc(&["include"]),

            // ---------- Matemática / física ----------
            Lib::new("glm", &gh("g-truc/glm"), "1.0.1", Header).inc(&["."]),
            Lib::new("bullet3", &gh("bulletphysics/bullet3"), "3.25", Cmake)
            .opts(&["-DBUILD_BULLET2_DEMOS=OFF", "-DBUILD_CPU_DEMOS=OFF",
                  "-DBUILD_OPENGL3_DEMOS=OFF", "-DBUILD_UNIT_TESTS=OFF",
                  "-DBUILD_EXTRAS=OFF", "-DBUILD_PYBULLET=OFF", "-DBUILD_ENET=OFF",
                  "-DBUILD_CLSOCKET=OFF", "-DINSTALL_LIBS=ON",
                  "-DUSE_MSVC_RUNTIME_LIBRARY_DLL=ON"])
            // ordem importa no link estático do Linux
            .libs(&["BulletDynamics", "BulletCollision", "LinearMath"])
            .inc(&["include/bullet"]),
            Lib::new("joltphysics", &gh("jrouwe/JoltPhysics"), "v5.2.0", Cmake)
            .subdir("Build")
            .opts(&["-DTARGET_UNIT_TESTS=OFF", "-DTARGET_HELLO_WORLD=OFF",
                  "-DTARGET_PERFORMANCE_TEST=OFF", "-DTARGET_SAMPLES=OFF",
                  "-DTARGET_VIEWER=OFF", "-DUSE_STATIC_MSVC_RUNTIME_LIBRARY=OFF",
                  "-DINTERPROCEDURAL_OPTIMIZATION=OFF"])
            .libs(&["Jolt"]),
            Lib::new("entt", &gh("skypjack/entt"), "v3.14.0", Header).inc(&["src"]),

            // ---------- Utilitários ----------
            Lib::new("fmt", &gh("fmtlib/fmt"), "11.0.2", Cmake)
            .opts(&["-DFMT_TEST=OFF", "-DFMT_DOC=OFF"]).libs(&["fmt"]),
            Lib::new("spdlog", &gh("gabime/spdlog"), "v1.14.1", Cmake)
            .opts(&["-DSPDLOG_BUILD_EXAMPLE=OFF", "-DSPDLOG_BUILD_TESTS=OFF",
                  "-DSPDLOG_FMT_EXTERNAL=OFF"])
            .libs(&["spdlog"]).defs(&["SPDLOG_COMPILED_LIB"]),
            Lib::new("cli11", &gh("CLIUtils/CLI11"), "v2.4.2", Header).inc(&["include"]),
            Lib::new("magic_enum", &gh("Neargye/magic_enum"), "v0.9.6", Header).inc(&["include"]),
            Lib::new("doctest", &gh("doctest/doctest"), "v2.4.11", Header).inc(&["."]),
            Lib::new("miniaudio", &gh("mackron/miniaudio"), "0.11.21", Header).inc(&["."]),
            Lib::new("freetype", &gh("freetype/freetype"), "VER-2-13-3", Cmake)
            .opts(&["-DFT_DISABLE_ZLIB=ON", "-DFT_DISABLE_BZIP2=ON", "-DFT_DISABLE_PNG=ON",
                  "-DFT_DISABLE_HARFBUZZ=ON", "-DFT_DISABLE_BROTLI=ON"])
            .libs(&["freetype"]).inc(&["include/freetype2"]),

            // ---------- UI ----------
            Lib::new("imgui", &gh("ocornut/imgui"), "v1.91.5", Sources)
            .srcs(&["imgui.cpp", "imgui_draw.cpp", "imgui_tables.cpp",
                  "imgui_widgets.cpp", "imgui_demo.cpp"])
            .inc(&["."]),
            Lib::new("imgui_glfw_gl3", &gh("ocornut/imgui"), "v1.91.5", Sources)
            .srcs(&["backends/imgui_impl_glfw.cpp", "backends/imgui_impl_opengl3.cpp"])
            .inc(&["backends"])
            .deps(&["imgui", "glfw", "opengl"]),
            Lib::new("imgui_glfw_vulkan", &gh("ocornut/imgui"), "v1.91.5", Sources)
            .srcs(&["backends/imgui_impl_glfw.cpp", "backends/imgui_impl_vulkan.cpp"])
            .inc(&["backends"]).defs(&["VK_NO_PROTOTYPES"])
            .deps(&["imgui", "glfw", "vulkan"]),
            Lib::new("imgui_sdl2_gl3", &gh("ocornut/imgui"), "v1.91.5", Sources)
            .srcs(&["backends/imgui_impl_sdl2.cpp", "backends/imgui_impl_opengl3.cpp"])
            .inc(&["backends"])
            .deps(&["imgui", "sdl2", "opengl"]),
            Lib::new("implot", &gh("epezent/implot"), "v0.16", Sources)
            .srcs(&["implot.cpp", "implot_items.cpp"]).inc(&["."]).deps(&["imgui"]),

            // ---------- SDKs proprietários (baixe à mão, veja README) ----------
            Lib::new("fmod", "", "", Prebuilt)
            .prebuilt("FMOD_SDK", "fmod", "api/core/lib/{arch}", "api/core/lib/{arch}",
                      "Baixe o FMOD Engine em https://www.fmod.com/download e extraia em redist/_sdk/fmod (ou defina FMOD_SDK)")
            .inc(&["api/core/inc"])
            .libs(&["fmod"]).libs_win(&["fmod_vc"]),
            Lib::new("fmodstudio", "", "", Prebuilt)
            .prebuilt("FMOD_SDK", "fmod", "api/studio/lib/{arch}", "api/studio/lib/{arch}",
                      "Baixe o FMOD Engine em https://www.fmod.com/download e extraia em redist/_sdk/fmod (ou defina FMOD_SDK)")
            .inc(&["api/studio/inc"])
            .libs(&["fmodstudio"]).libs_win(&["fmodstudio_vc"])
            .deps(&["fmod"]),
            Lib::new("steamworks", "", "", Prebuilt)
            .prebuilt("STEAMWORKS_SDK", "steamworks", "redistributable_bin/linux64", "redistributable_bin/win64",
                      "Baixe o Steamworks SDK (partner.steamgames.com) e extraia em redist/_sdk/steamworks (ou defina STEAMWORKS_SDK)")
            .inc(&["public"])
            .libs(&["steam_api"]).libs_win(&["steam_api64"]),

            // ---------- Áudio ----------
            Lib::new("openal", &gh("kcat/openal-soft"), "1.23.1", Cmake).shared()
            .opts(&["-DALSOFT_UTILS=OFF", "-DALSOFT_EXAMPLES=OFF", "-DALSOFT_TESTS=OFF",
                  "-DALSOFT_INSTALL_EXAMPLES=OFF", "-DALSOFT_INSTALL_UTILS=OFF", "-DLIBTYPE=SHARED"])
            .libs(&["openal"]).libs_win(&["OpenAL32"]),
            Lib::new("dr_libs", &gh("mackron/dr_libs"), "master", Header).inc(&["."]),

            // ---------- Gráficos / jogos ----------
            Lib::new("raylib", &gh("raysan5/raylib"), "5.5", Cmake)
            .opts(&["-DBUILD_EXAMPLES=OFF", "-DBUILD_GAMES=OFF"])
            .libs(&["raylib"])
            .sys_linux(&["GL", "pthread", "dl", "m"])
            .sys_win(&["opengl32", "gdi32", "winmm", "user32", "shell32"]),
            Lib::new("volk", &gh("zeux/volk"), "vulkan-sdk-1.3.296.0", Sources)
            .srcs(&["volk.c"]).inc(&["."]).sys_linux(&["dl"]).deps(&["vulkan_headers"]),
            Lib::new("meshoptimizer", &gh("zeux/meshoptimizer"), "v0.22", Cmake)
            .opts(&["-DMESHOPT_BUILD_DEMO=OFF", "-DMESHOPT_BUILD_GLTFPACK=OFF"])
            .libs(&["meshoptimizer"]),
            Lib::new("tracy", &gh("wolfpld/tracy"), "v0.11.1", Sources)
            .srcs(&["public/TracyClient.cpp"]).inc(&["public"]).defs(&["TRACY_ENABLE"])
            .sys_linux(&["pthread", "dl"]).sys_win(&["ws2_32", "dbghelp"]),

            // ---------- Física / jogos ----------
            Lib::new("reactphysics3d", &gh("DanielChappuis/reactphysics3d"), "v0.10.2", Cmake)
            .opts(&["-DRP3D_COMPILE_TESTBED=OFF", "-DRP3D_COMPILE_TESTS=OFF", "-DRP3D_CODE_COVERAGE_ENABLED=OFF"])
            .libs(&["reactphysics3d"]),
            Lib::new("enet", &gh("lsalzman/enet"), "v1.3.18", Cmake)
            .libs(&["enet"]).sys_win(&["ws2_32", "winmm"]),

            // ---------- Matemática ----------
            Lib::new("eigen", "https://gitlab.com/libeigen/eigen.git", "3.4.0", Header).inc(&["."]),
            Lib::new("cglm", &gh("recp/cglm"), "v0.9.4", Header).inc(&["include"]),

            // ---------- Scripting ----------
            Lib::new("lua", &gh("lua/lua"), "v5.4.7", Sources)
            .srcs(&["lapi.c", "lcode.c", "lctype.c", "ldebug.c", "ldo.c", "ldump.c", "lfunc.c",
                  "lgc.c", "llex.c", "lmem.c", "lobject.c", "lopcodes.c", "lparser.c", "lstate.c",
                  "lstring.c", "ltable.c", "ltm.c", "lundump.c", "lvm.c", "lzio.c", "lauxlib.c",
                  "lbaselib.c", "lcorolib.c", "ldblib.c", "liolib.c", "lmathlib.c", "loadlib.c",
                  "loslib.c", "lstrlib.c", "ltablib.c", "lutf8lib.c", "linit.c"])
            .inc(&["."]).sys_linux(&["m", "dl"]),
            Lib::new("sol2", &gh("ThePhD/sol2"), "v3.3.0", Header).inc(&["include"]).deps(&["lua"]),

            // ---------- Dados / texto ----------
            Lib::new("pugixml", &gh("zeux/pugixml"), "v1.14", Cmake).libs(&["pugixml"]),
            Lib::new("tinyxml2", &gh("leethomason/tinyxml2"), "10.0.0", Cmake)
            .opts(&["-Dtinyxml2_BUILD_TESTING=OFF"]).libs(&["tinyxml2"]),
            Lib::new("tomlplusplus", &gh("marzer/tomlplusplus"), "v3.4.0", Header).inc(&["include"]),

            // ---------- Utilitários C++ ----------
            Lib::new("range_v3", &gh("ericniebler/range-v3"), "0.12.0", Header).inc(&["include"]),
            Lib::new("unordered_dense", &gh("martinus/unordered_dense"), "v4.4.0", Header).inc(&["include"]),
            Lib::new("concurrentqueue", &gh("cameron314/concurrentqueue"), "v1.0.4", Header).inc(&["."]),
            Lib::new("asio", &gh("chriskohlhoff/asio"), "asio-1-30-2", Header)
            .inc(&["asio/include"]).defs(&["ASIO_STANDALONE"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32", "mswsock"]),

            // ---------- Testes ----------
            Lib::new("catch2", &gh("catchorg/Catch2"), "v3.7.1", Cmake)
            .opts(&["-DCATCH_INSTALL_DOCS=OFF", "-DCATCH_INSTALL_EXTRAS=OFF", "-DCATCH_DEVELOPMENT_BUILD=OFF"])
            .libs(&["Catch2Main", "Catch2"]),
            Lib::new("googletest", &gh("google/googletest"), "v1.15.2", Cmake)
            .opts(&["-DINSTALL_GTEST=ON", "-Dgtest_force_shared_crt=ON", "-DBUILD_GMOCK=ON"])
            .libs(&["gmock_main", "gmock", "gtest"]).sys_linux(&["pthread"]),

            // ================= NOVAS =================

            // ---------- OpenGL loader ----------
            // glad v1 (gera o loader no build; exige python no PATH). Uso: #include <glad/glad.h>
            Lib::new("glad", &gh("Dav1dde/glad"), "v0.1.36", Cmake)
            .opts(&["-DGLAD_INSTALL=ON", "-DGLAD_PROFILE=core", "-DGLAD_API=gl=3.3",
                  "-DGLAD_GENERATOR=c", "-DGLAD_EXTENSIONS="])
            // GLFW_INCLUDE_NONE: o glfw3.h não inclui GL/gl.h, então a ordem glad/glfw deixa de importar
            .libs(&["glad"]).defs(&["GLFW_INCLUDE_NONE"]).sys_linux(&["dl"]),

            // ---------- Aliases (mesmo repo, outro nome de `#include`) ----------
            Lib::new("stb_image", &gh("nothings/stb"), "master", Header).inc(&["."]),
            Lib::new("tinyxml", &gh("leethomason/tinyxml2"), "10.0.0", Cmake)
            .opts(&["-Dtinyxml2_BUILD_TESTING=OFF"]).libs(&["tinyxml2"]),
            Lib::new("toml", &gh("marzer/tomlplusplus"), "v3.4.0", Header).inc(&["include"]),

            // ---------- Janela / multimídia ----------
            Lib::new("sfml", &gh("SFML/SFML"), "3.0.0", Cmake).shared()
            .opts(&["-DSFML_BUILD_EXAMPLES=OFF", "-DSFML_BUILD_DOC=OFF", "-DSFML_BUILD_TEST_SUITE=OFF"])
            .libs(&["sfml-graphics", "sfml-audio", "sfml-network", "sfml-window", "sfml-system"])
            .inc(&["include"]),
            Lib::new("sdl2_image", &gh("libsdl-org/SDL_image"), "release-2.8.2", Cmake).shared()
            .opts(&["-DSDL2IMAGE_SAMPLES=OFF", "-DSDL2IMAGE_TESTS=OFF"])
            .libs(&["SDL2_image"]).deps(&["sdl2"]),
            Lib::new("sdl2_ttf", &gh("libsdl-org/SDL_ttf"), "release-2.22.0", Cmake).shared()
            .opts(&["-DSDL2TTF_SAMPLES=OFF", "-DSDL2TTF_VENDORED=ON"])
            .libs(&["SDL2_ttf"]).deps(&["sdl2"]),
            Lib::new("sdl2_mixer", &gh("libsdl-org/SDL_mixer"), "release-2.8.0", Cmake).shared()
            .opts(&["-DSDL2MIXER_SAMPLES=OFF", "-DSDL2MIXER_VENDORED=ON"])
            .libs(&["SDL2_mixer"]).deps(&["sdl2"]),

            // ---------- Modelos 3D / engines ----------
            Lib::new("assimp", &gh("assimp/assimp"), "v5.4.3", Cmake).shared()
            .opts(&["-DASSIMP_BUILD_TESTS=OFF", "-DASSIMP_BUILD_ASSIMP_TOOLS=OFF",
                  "-DASSIMP_BUILD_SAMPLES=OFF", "-DASSIMP_INSTALL=ON",
                  "-DASSIMP_WARNINGS_AS_ERRORS=OFF"])
            .libs(&["assimp"]).libs_win(&["assimp-vc143-mt"]),
            // Ogre3D: build pesado (baixa dependências próprias). Plugins/recursos ficam em redist/<plat>/<arch>/lib/OGRE.
            Lib::new("ogre", &gh("OGRECave/ogre"), "v14.3.2", Cmake).shared()
            .opts(&["-DOGRE_BUILD_SAMPLES=OFF", "-DOGRE_BUILD_TOOLS=OFF", "-DOGRE_BUILD_TESTS=OFF",
                  "-DOGRE_INSTALL_SAMPLES=OFF", "-DOGRE_INSTALL_DOCS=OFF",
                  "-DOGRE_BUILD_DEPENDENCIES=ON", "-DOGRE_BUILD_COMPONENT_PYTHON=OFF",
                  "-DOGRE_BUILD_COMPONENT_JAVA=OFF", "-DOGRE_BUILD_COMPONENT_CSHARP=OFF"])
            .libs(&["OgreBites", "OgreOverlay", "OgreRTShaderSystem", "OgreMain"])
            .inc(&["include/OGRE", "include/OGRE/Bites", "include/OGRE/Overlay", "include/OGRE/RTShaderSystem"])
            .sys_linux(&["pthread", "dl"]),
            Lib::new("nanogui", &gh("wjakob/nanogui"), "master", Cmake).shared()
            .opts(&["-DNANOGUI_BUILD_EXAMPLE=OFF", "-DNANOGUI_BUILD_PYTHON=OFF",
                  "-DNANOGUI_INSTALL=ON"])
            .libs(&["nanogui"]).inc(&["include"])
            .deps(&["eigen"]).sys_linux(&["GL", "pthread", "dl"]),
            Lib::new("box2d", &gh("erincatto/box2d"), "v2.4.2", Cmake)
            .opts(&["-DBOX2D_BUILD_UNIT_TESTS=OFF", "-DBOX2D_BUILD_TESTBED=OFF"])
            .libs(&["box2d"]).inc(&["include"]),

            // ---------- Bancos de dados ----------
            Lib::new("sqlitecpp", &gh("SRombauts/SQLiteCpp"), "3.3.2", Cmake)
            .opts(&["-DSQLITECPP_RUN_CPPLINT=OFF", "-DSQLITECPP_RUN_CPPCHECK=OFF",
                  "-DSQLITECPP_BUILD_TESTS=OFF", "-DSQLITECPP_BUILD_EXAMPLES=OFF",
                  "-DSQLITECPP_INTERNAL_SQLITE=ON"])
            .libs(&["SQLiteCpp", "sqlite3"]).sys_linux(&["pthread", "dl"]),
            // mysql++: lib do sistema (sudo apt install libmysql++-dev). Includes: <mysql++/mysql++.h>
            Lib::new("mysqlpp", "", "", System)
            .libs(&["mysqlpp", "mysqlclient"]).libs_win(&["mysqlpp", "libmysql"])
            .inc(&["/usr/include/mysql"]),
            // QTL: wrapper C++ sobre clientes MySQL/SQLite/ODBC (precisa de mysqlpp/sqlite3 conforme o uso)
            Lib::new("qtl", &gh("qicosmos/qtl"), "master", Header).inc(&["include"]),

            // ---------- Rede / eventos / sistema ----------
            Lib::new("eventpp", &gh("wqking/eventpp"), "v0.1.3", Header).inc(&["include"]),
            Lib::new("httplib", &gh("yhirose/cpp-httplib"), "v0.18.1", Header).inc(&["."])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32", "crypt32"]),
            // gpsd: libgps do sistema (sudo apt install libgps-dev). Só Linux. Uso: #include <gps.h>
            Lib::new("gpsd", "", "", System).libs(&["gps"]).sys_linux(&["m", "pthread"]),

            // ---------- Terminal / TUI ----------
            // ncurses: no Linux usa ncursesw (UTF-8 / wide). Pacote: libncurses-dev ou libncursesw5-dev.
            // No Windows o rsfmake troca automaticamente `ncurses`/`ncursesw` por `pdcurses`.
            // Uso: #include <ncurses.h> (headers do ncursesw são compatíveis).
            Lib::new("ncurses", "", "", System).libs(&["ncursesw"]),
            // Alias explícito (mesmo link). Preferível se o projeto já declara ncursesw.
            Lib::new("ncursesw", "", "", System).libs(&["ncursesw"]),
            // PDCursesMod (porta wincon): ncurses para o terminal do Windows. Compilado junto com o projeto
            // (Sources, sem CMake). O rsfmake gera redist/windows/<arch>/include/ncurses.h que inclui <curses.h>.
            // Só Windows (também no cross com MinGW).
            Lib::new("pdcurses", &gh("Bill-Gray/PDCursesMod"), "v4.4.0", Sources)
            .srcs(&["pdcurses/*.c", "wincon/*.c"])
            .inc(&[".", "wincon"])
            .defs(&["PDC_WIDE", "PDC_FORCE_UTF8", "HAVE_VSNPRINTF", "_CRT_SECURE_NO_WARNINGS"])
            .sys_win(&["user32", "gdi32", "advapi32", "shell32", "ole32", "comdlg32", "winspool", "winmm"]),

            // ---------- Matemática / científico ----------
            // Armadillo precisa de BLAS/LAPACK (sudo apt install libopenblas-dev).
            Lib::new("armadillo", "https://gitlab.com/conradsnicta/armadillo-code.git", "14.0.3", Header)
            .inc(&["include"]).defs(&["ARMA_DONT_USE_WRAPPER"])
            .sys_linux(&["openblas"]).sys_win(&["openblas"]),

            // ---------- Utilitários extras ----------
            Lib::new("zlib", &gh("madler/zlib"), "v1.3.1", Cmake)
            .opts(&["-DZLIB_BUILD_EXAMPLES=OFF"]).libs(&["z"]).libs_win(&["zlibstatic"]),
            Lib::new("yaml_cpp", &gh("jbeder/yaml-cpp"), "0.8.0", Cmake)
            .opts(&["-DYAML_CPP_BUILD_TESTS=OFF", "-DYAML_CPP_BUILD_TOOLS=OFF",
                  "-DYAML_CPP_BUILD_CONTRIB=OFF"])
            .libs(&["yaml-cpp"]),
      ];

      list.into_iter().map(|l| (l.name.clone(), l)).collect()
}