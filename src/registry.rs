//! Registro embutido de bibliotecas.
//!
//! Tipos de lib (Kind):
//!   Cmake   -> clona, compila com CMake+Ninja e instala em redist/<plat>/<arch>
//!              (`inc` é relativo ao prefixo de instalação)
//!   Header  -> só clona; `inc` é relativo ao diretório do código-fonte
//!   Sources -> clona e compila os `srcs` junto com o projeto (ex: imgui)
//!   System  -> lib do sistema (opengl, vulkan), nada é baixado
//!   Sources -> aceita globs em `srcs` (ex: "pdcurses/*.c")
//!   apt     -> (qualquer tipo) pacotes -dev do Debian/Ubuntu; o rsfmake confere com dpkg
//!              e oferece instalar com `sudo apt-get install` antes de compilar (só no Linux).
//!   sys_inc -> includes absolutos extras do sistema (ex: /usr/include/jsoncpp)
//!   win_sdk -> Windows: libs que no Linux vêm do apt e no Windows precisam de instalação manual.
//!              O rsfmake lê a variável de ambiente (ex: FFMPEG_DIR), usa <dir>/include e <dir>/lib,
//!              copia as DLLs de <dir>/bin e, se faltar, para com uma mensagem explicando o que instalar.
//!   deps_win / linux_only -> deps só do Windows (ex: jsoncpp p/ drogon) / lib sem versão Windows (gpsd)
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
      /// dependências extras só no Windows (no Linux vêm do sistema via apt)
      pub deps_win: Vec<String>,
      pub srcs: Vec<String>,
      pub shared: bool,
      pub sdk_env: String,
      pub sdk_dir: String,
      pub libdir_linux: String,
      pub libdir_win: String,
      pub help: String,
      /// pacotes -dev do sistema (Debian/Ubuntu) necessários no Linux, ex: "libpq-dev"
      pub apt: Vec<String>,
      /// diretórios de include absolutos do sistema (ex: /usr/include/jsoncpp)
      pub sys_inc: Vec<String>,
      /// Windows: variável(is) de ambiente (separadas por vírgula) com a pasta raiz da lib instalada
      /// à mão (contendo include/ e lib/). Vazio = a lib não precisa de nada no Windows.
      pub win_env: String,
      /// Windows: includes extras relativos à raiz de `win_env` (ex: include/libusb-1.0)
      pub win_inc: Vec<String>,
      /// Windows: texto de ajuda mostrado quando `win_env` não está definida
      pub win_help: String,
      /// lib que só existe no Linux (ex: gpsd)
      pub linux_only: bool,
      /// Windows: URLs (zip) tentadas em ordem para baixar a lib sozinho quando `win_env` não está definida
      pub win_zip: Vec<String>,
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
      fn deps_win(mut self, x: &[&str]) -> Self { self.deps_win = v(x); self }
      fn srcs(mut self, x: &[&str]) -> Self { self.srcs = v(x); self }
      fn apt(mut self, x: &[&str]) -> Self { self.apt = v(x); self }
      fn sys_inc(mut self, x: &[&str]) -> Self { self.sys_inc = v(x); self }
      fn subdir(mut self, x: &str) -> Self { self.subdir = x.into(); self }
      fn win_sdk(mut self, env: &str, help: &str) -> Self { self.win_env = env.into(); self.win_help = help.into(); self }
      fn win_inc(mut self, x: &[&str]) -> Self { self.win_inc = v(x); self }
      fn linux_only(mut self) -> Self { self.linux_only = true; self }
      fn win_zip(mut self, x: &[&str]) -> Self { self.win_zip = v(x); self }
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
            Lib::new("opengl", "", "", System).libs(&["GL"]).libs_win(&["opengl32"])
            .apt(&["libgl-dev"]),
            Lib::new("vulkan", "", "", System).libs(&["vulkan"]).libs_win(&["vulkan-1"])
            .apt(&["libvulkan-dev"])
            .win_sdk("VULKAN_SDK", "Install the LunarG Vulkan SDK (https://vulkan.lunarg.com) and restart the terminal so VULKAN_SDK is set"),

            // ---------- Janela / input ----------
            Lib::new("glfw", &gh("glfw/glfw"), "3.4", Cmake)
            .opts(&["-DGLFW_BUILD_EXAMPLES=OFF", "-DGLFW_BUILD_TESTS=OFF",
                  "-DGLFW_BUILD_DOCS=OFF", "-DGLFW_INSTALL=ON"])
            .libs(&["glfw3"])
            .sys_linux(&["pthread", "dl", "m"])
            .sys_win(&["gdi32", "user32", "shell32"])
            .apt(&["libgl-dev", "libx11-dev", "libxrandr-dev", "libxinerama-dev", "libxcursor-dev", "libxi-dev", "libwayland-dev", "libxkbcommon-dev", "wayland-protocols", "pkg-config"]),
            Lib::new("sdl2", &gh("libsdl-org/SDL"), "release-2.30.9", Cmake).shared()
            .opts(&["-DSDL_SHARED=ON", "-DSDL_STATIC=OFF", "-DSDL_TEST=OFF"])
            .libs(&["SDL2"]).libs_win(&["SDL2main", "SDL2"])
            .inc(&["include/SDL2"])
            .apt(&["libgl-dev", "libx11-dev", "libxext-dev", "libxrandr-dev", "libxcursor-dev", "libxi-dev", "libxfixes-dev", "libxss-dev", "libxkbcommon-dev", "libasound2-dev"]),
            Lib::new("sdl3", &gh("libsdl-org/SDL"), "release-3.2.0", Cmake).shared()
            .opts(&["-DSDL_SHARED=ON", "-DSDL_STATIC=OFF", "-DSDL_TESTS=OFF",
                  "-DSDL_EXAMPLES=OFF"])
            .libs(&["SDL3"])
            .apt(&["libgl-dev", "libx11-dev", "libxext-dev", "libxrandr-dev", "libxcursor-dev", "libxi-dev", "libxfixes-dev", "libxss-dev", "libxkbcommon-dev", "libasound2-dev"]),

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
                      "Download the FMOD Engine from https://www.fmod.com/download and extract it into redist/_sdk/fmod (or set FMOD_SDK)")
            .inc(&["api/core/inc"])
            .libs(&["fmod"]).libs_win(&["fmod_vc"]),
            Lib::new("fmodstudio", "", "", Prebuilt)
            .prebuilt("FMOD_SDK", "fmod", "api/studio/lib/{arch}", "api/studio/lib/{arch}",
                      "Download the FMOD Engine from https://www.fmod.com/download and extract it into redist/_sdk/fmod (or set FMOD_SDK)")
            .inc(&["api/studio/inc"])
            .libs(&["fmodstudio"]).libs_win(&["fmodstudio_vc"])
            .deps(&["fmod"]),
            Lib::new("steamworks", "", "", Prebuilt)
            .prebuilt("STEAMWORKS_SDK", "steamworks", "redistributable_bin/linux64", "redistributable_bin/win64",
                      "Download the Steamworks SDK (partner.steamgames.com) and extract it into redist/_sdk/steamworks (or set STEAMWORKS_SDK)")
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
            .sys_win(&["opengl32", "gdi32", "winmm", "user32", "shell32"])
            .apt(&["libgl-dev", "libx11-dev", "libxrandr-dev", "libxinerama-dev", "libxcursor-dev", "libxi-dev"]),
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
            .inc(&["include"])
            .apt(&["libgl-dev", "libx11-dev", "libxrandr-dev", "libxcursor-dev", "libxi-dev", "libudev-dev", "libfreetype-dev", "libopenal-dev", "libvorbis-dev", "libflac-dev"]),
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
            .deps(&["eigen"]).sys_linux(&["GL", "pthread", "dl"])
            .apt(&["libgl-dev", "libx11-dev", "libxrandr-dev", "libxinerama-dev", "libxcursor-dev", "libxi-dev"]),
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
            .inc(&["/usr/include/mysql"])
            .apt(&["libmysql++-dev"])
            .win_sdk("MYSQLPP_DIR", "Build/install mysql++ together with MySQL Connector/C and set MYSQLPP_DIR to a folder with include/ (mysql++/ and mysql.h) and lib/ (mysqlpp.lib, libmysql.lib)")
            .win_inc(&["include/mysql"]),
            // QTL: wrapper C++ sobre clientes MySQL/SQLite/ODBC (precisa de mysqlpp/sqlite3 conforme o uso)
            Lib::new("qtl", &gh("qicosmos/qtl"), "master", Header).inc(&["include"]),

            // ---------- Rede / eventos / sistema ----------
            Lib::new("eventpp", &gh("wqking/eventpp"), "v0.1.3", Header).inc(&["include"]),
            Lib::new("httplib", &gh("yhirose/cpp-httplib"), "v0.18.1", Header).inc(&["."])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32", "crypt32"]),
            // gpsd: libgps do sistema (sudo apt install libgps-dev). Só Linux. Uso: #include <gps.h>
            Lib::new("gpsd", "", "", System).libs(&["gps"]).sys_linux(&["m", "pthread"])
            .apt(&["libgps-dev"]).linux_only(),

            // ---------- Terminal / TUI ----------
            // ncurses: no Linux usa ncursesw (UTF-8 / wide). Pacote: libncurses-dev ou libncursesw5-dev.
            // No Windows o rsfmake troca automaticamente `ncurses`/`ncursesw` por `pdcurses`.
            // Uso: #include <ncurses.h> (headers do ncursesw são compatíveis).
            Lib::new("ncurses", "", "", System).libs(&["ncursesw"])
            .apt(&["libncurses-dev"]),
            // Alias explícito (mesmo link). Preferível se o projeto já declara ncursesw.
            Lib::new("ncursesw", "", "", System).libs(&["ncursesw"])
            .apt(&["libncurses-dev"]),
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
            .sys_linux(&["openblas"]).sys_win(&["libopenblas"])
            .apt(&["libopenblas-dev"])
            .win_sdk("OPENBLAS_DIR", "Download the Windows OpenBLAS release (github.com/OpenMathLib/OpenBLAS/releases, OpenBLAS-x.y.z-x64.zip), extract it and set OPENBLAS_DIR to that folder (lib/libopenblas.lib, bin/libopenblas.dll)")
            .win_zip(&["https://github.com/OpenMathLib/OpenBLAS/releases/download/v0.3.28/OpenBLAS-0.3.28-x64.zip"]),

            // ---------- Utilitários extras ----------
            Lib::new("zlib", &gh("madler/zlib"), "v1.3.1", Cmake)
            .opts(&["-DZLIB_BUILD_EXAMPLES=OFF"]).libs(&["z"]).libs_win(&["zlibstatic"]),
            Lib::new("yaml_cpp", &gh("jbeder/yaml-cpp"), "0.8.0", Cmake)
            .opts(&["-DYAML_CPP_BUILD_TESTS=OFF", "-DYAML_CPP_BUILD_TOOLS=OFF",
                  "-DYAML_CPP_BUILD_CONTRIB=OFF"])
            .libs(&["yaml-cpp"]),

            // ================= NOVAS (libs_to_add) =================
            // Obs.: tags/nomes de libs de link foram escritos sem teste de build; ajuste se alguma falhar.

            // ---------- Gráficos / shaders ----------
            // bgfx.cmake (traz bx, bimg e bgfx como submódulos). Linux: libx11-dev, libgl-dev.
            Lib::new("bgfx", &gh("bkaradzic/bgfx.cmake"), "master", Cmake)
            .opts(&["-DBGFX_BUILD_EXAMPLES=OFF", "-DBGFX_BUILD_TOOLS=OFF", "-DBGFX_INSTALL=ON"])
            .libs(&["bgfx", "bimg_decode", "bimg", "bx"]).inc(&["include"])
            .sys_linux(&["GL", "X11", "pthread", "dl", "rt"]).sys_win(&["psapi", "gdi32", "user32"])
            .apt(&["libgl-dev", "libx11-dev"]),
            Lib::new("spirv_cross", &gh("KhronosGroup/SPIRV-Cross"), "vulkan-sdk-1.3.296.0", Cmake)
            .opts(&["-DSPIRV_CROSS_CLI=OFF", "-DSPIRV_CROSS_ENABLE_TESTS=OFF"])
            .libs(&["spirv-cross-reflect", "spirv-cross-util", "spirv-cross-cpp", "spirv-cross-msl",
                  "spirv-cross-hlsl", "spirv-cross-glsl", "spirv-cross-core"])
            .inc(&["include/spirv_cross"]),
            Lib::new("ktx", &gh("KhronosGroup/KTX-Software"), "v4.3.2", Cmake)
            .opts(&["-DKTX_FEATURE_TESTS=OFF", "-DKTX_FEATURE_TOOLS=OFF", "-DKTX_FEATURE_LOADTEST_APPS=OFF",
                  "-DKTX_FEATURE_STATIC_LIBRARY=ON"])
            .libs(&["ktx"]).inc(&["include"]).sys_linux(&["pthread", "dl"]),
            Lib::new("rmlui", &gh("mikke89/RmlUi"), "6.0", Cmake)
            .opts(&["-DBUILD_SAMPLES=OFF", "-DBUILD_TESTING=OFF"])
            .libs(&["RmlCore"]).inc(&["include"]).deps(&["freetype"]),
            Lib::new("harfbuzz", &gh("harfbuzz/harfbuzz"), "10.0.1", Cmake)
            .opts(&["-DHB_BUILD_TESTS=OFF", "-DHB_BUILD_UTILS=OFF", "-DHB_BUILD_SUBSET=OFF",
                  "-DHB_HAVE_FREETYPE=ON"])
            .libs(&["harfbuzz"]).inc(&["include/harfbuzz"]).deps(&["freetype"]),
            Lib::new("imgui_node_editor", &gh("thedmd/imgui-node-editor"), "master", Sources)
            .srcs(&["crude_json.cpp", "imgui_canvas.cpp", "imgui_node_editor.cpp", "imgui_node_editor_api.cpp"])
            .inc(&["."]).deps(&["imgui"]),
            Lib::new("imguizmo", &gh("CedricGuillemet/ImGuizmo"), "master", Sources)
            .srcs(&["ImGuizmo.cpp"]).inc(&["."]).deps(&["imgui"]),
            Lib::new("optick", &gh("bombomby/optick"), "master", Sources)
            .srcs(&["src/optick_core.cpp", "src/optick_message.cpp", "src/optick_miniz.cpp",
                  "src/optick_serialization.cpp", "src/optick_server.cpp", "src/optick_capi.cpp",
                  "src/optick_gpu.cpp"])
            .inc(&["src"])
            .defs(&["USE_OPTICK=1", "OPTICK_ENABLE_GPU_D3D12=0", "OPTICK_ENABLE_GPU_VULKAN=0"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32"]),

            // ---------- Áudio ----------
            // SoLoud com backend miniaudio (sem dependências externas)
            Lib::new("soloud", &gh("jarikomppa/soloud"), "RELEASE_20200207", Sources)
            .srcs(&["src/core/*.cpp", "src/filter/*.cpp", "src/audiosource/wav/*.c*",
                  "src/backend/miniaudio/*.cpp"])
            .inc(&["include"]).defs(&["WITH_MINIAUDIO"])
            .sys_linux(&["pthread", "dl", "m"]).sys_win(&["winmm"]),
            Lib::new("libsndfile", &gh("libsndfile/libsndfile"), "1.2.2", Cmake)
            .opts(&["-DBUILD_PROGRAMS=OFF", "-DBUILD_EXAMPLES=OFF", "-DBUILD_TESTING=OFF",
                  "-DBUILD_REGTEST=OFF", "-DENABLE_EXTERNAL_LIBS=OFF", "-DENABLE_MPEG=OFF",
                  "-DINSTALL_MANPAGES=OFF"])
            .libs(&["sndfile"]).inc(&["include"]).sys_linux(&["m"]),
            Lib::new("rtaudio", &gh("thestk/rtaudio"), "6.0.1", Cmake)
            .opts(&["-DRTAUDIO_BUILD_TESTING=OFF", "-DRTAUDIO_BUILD_SHARED_LIBS=OFF",
                  "-DRTAUDIO_BUILD_STATIC_LIBS=ON",
                  // ALSA only on Linux: avoids auto-enabling PulseAudio/JACK/OSS when their -dev packages happen to be installed
                  "-DRTAUDIO_API_PULSE=OFF", "-DRTAUDIO_API_JACK=OFF", "-DRTAUDIO_API_OSS=OFF"])
            .libs(&["rtaudio"]).inc(&["include/rtaudio"]).defs(&["RTAUDIO_STATIC"])
            .sys_linux(&["asound", "pthread"]).apt(&["libasound2-dev"])
            .sys_win(&["ole32", "winmm", "ksuser", "mfplat", "mfuuid", "wmcodecdspuuid", "dsound"]),
            Lib::new("libogg", &gh("xiph/ogg"), "v1.3.5", Cmake)
            .opts(&["-DINSTALL_DOCS=OFF", "-DBUILD_TESTING=OFF"])
            .libs(&["ogg"]).inc(&["include"]),
            Lib::new("libvorbis", &gh("xiph/vorbis"), "v1.3.7", Cmake)
            .opts(&["-DBUILD_TESTING=OFF"])
            .libs(&["vorbisfile", "vorbisenc", "vorbis"]).inc(&["include"])
            .deps(&["libogg"]).sys_linux(&["m"]),
            // alias com o nome exatamente como estava em libs_to_add.txt
            Lib::new("lilbvorbis", &gh("xiph/vorbis"), "v1.3.7", Cmake)
            .opts(&["-DBUILD_TESTING=OFF"])
            .libs(&["vorbisfile", "vorbisenc", "vorbis"]).inc(&["include"])
            .deps(&["libogg"]).sys_linux(&["m"]),
            Lib::new("opus", &gh("xiph/opus"), "v1.5.2", Cmake)
            .opts(&["-DOPUS_BUILD_PROGRAMS=OFF", "-DOPUS_BUILD_TESTING=OFF"])
            .libs(&["opus"]).inc(&["include"]).sys_linux(&["m"]),
            // ffmpeg: libs do sistema (sudo apt install libavformat-dev libavcodec-dev libswscale-dev libswresample-dev).
            Lib::new("ffmpeg", "", "", System)
            .libs(&["avformat", "avcodec", "swscale", "swresample", "avutil"])
            .sys_linux(&["m", "pthread", "z"])
            .apt(&["libavformat-dev", "libavcodec-dev", "libavutil-dev", "libswscale-dev", "libswresample-dev"])
            .win_sdk("FFMPEG_DIR", "Download a *shared* Windows build (e.g. github.com/BtbN/FFmpeg-Builds, win64-gpl-shared), extract it and set FFMPEG_DIR to that folder (it has include/, lib/ and bin/)")
            .win_zip(&["https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl-shared.zip"]),

            // ---------- Rede / servidores web ----------
            // curl: no Linux precisa de libssl-dev; no Windows usa Schannel.
            Lib::new("curl", &gh("curl/curl"), "curl-8_10_1", Cmake).shared()
            .opts(&["-DBUILD_CURL_EXE=OFF", "-DBUILD_TESTING=OFF", "-DCURL_USE_LIBPSL=OFF",
                  "-DCURL_DISABLE_LDAP=ON"])
            .libs(&["curl"]).libs_win(&["libcurl_imp"]).inc(&["include"])
            .apt(&["libssl-dev"]),
            Lib::new("cpr", &gh("libcpr/cpr"), "1.11.0", Cmake)
            .opts(&["-DCPR_BUILD_TESTS=OFF", "-DCPR_USE_SYSTEM_CURL=ON"])
            .libs(&["cpr"]).inc(&["include"]).deps(&["curl"]),
            Lib::new("websocketpp", &gh("zaphoyd/websocketpp"), "0.8.2", Header).inc(&["."])
            .defs(&["ASIO_STANDALONE", "_WEBSOCKETPP_CPP11_STL_"]).deps(&["asio"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32", "mswsock"]),
            Lib::new("ixwebsocket", &gh("machinezone/IXWebSocket"), "v11.4.5", Cmake)
            .opts(&["-DUSE_TLS=OFF", "-DUSE_ZLIB=OFF", "-DUSE_TEST=OFF"])
            .libs(&["ixwebsocket"]).inc(&["include"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32", "shlwapi", "crypt32"]),
            Lib::new("crow", &gh("CrowCpp/Crow"), "v1.2.0", Header).inc(&["include"]).deps(&["asio"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32", "mswsock"]),
            // drogon: Linux needs libjsoncpp-dev and uuid-dev.
            // TLS/OpenSSL is disabled: drogon 1.9.7's trantor does not compile against OpenSSL 4.x
            // (-Werror on deprecated/const API). That means no HTTPS in the built-in server.
            Lib::new("drogon", &gh("drogonframework/drogon"), "v1.9.7", Cmake)
            .opts(&["-DBUILD_EXAMPLES=OFF", "-DBUILD_CTL=OFF", "-DBUILD_ORM=OFF", "-DBUILD_TESTING=OFF",
                  "-DCMAKE_DISABLE_FIND_PACKAGE_OpenSSL=ON", "-DCMAKE_DISABLE_FIND_PACKAGE_Botan=ON",
                  // optional features are off so the link line stays fixed (otherwise they depend on what is installed)
                  "-DCMAKE_DISABLE_FIND_PACKAGE_Brotli=ON", "-DCMAKE_DISABLE_FIND_PACKAGE_c-ares=ON",
                  "-DCMAKE_DISABLE_FIND_PACKAGE_Hiredis=ON", "-DCMAKE_DISABLE_FIND_PACKAGE_yaml-cpp=ON"])
            .libs(&["drogon", "trantor"]).inc(&["include"]).deps(&["zlib"])
            // Windows has no libjsoncpp-dev: build jsoncpp from source (installed in the same prefix,
            // which is already in CMAKE_PREFIX_PATH, so drogon's FindJsoncpp finds it).
            .deps_win(&["jsoncpp"])
            .sys_linux(&["jsoncpp", "uuid", "pthread", "dl"])
            .apt(&["libjsoncpp-dev", "uuid-dev"]).sys_inc(&["/usr/include/jsoncpp"])
            .sys_win(&["ws2_32", "rpcrt4", "crypt32", "iphlpapi", "shlwapi", "advapi32"]),
            // jsoncpp: only used as a build dependency of drogon on Windows (static, header at include/json/json.h).
            Lib::new("jsoncpp", &gh("open-source-parsers/jsoncpp"), "1.9.6", Cmake)
            .opts(&["-DJSONCPP_WITH_TESTS=OFF", "-DJSONCPP_WITH_POST_BUILD_UNITTEST=OFF",
                  "-DJSONCPP_WITH_EXAMPLE=OFF", "-DJSONCPP_WITH_PKGCONFIG_SUPPORT=OFF",
                  "-DBUILD_STATIC_LIBS=ON", "-DBUILD_OBJECT_LIBS=OFF"])
            .libs(&["jsoncpp"]).inc(&["include"]),
            // libusb: lib do sistema (sudo apt install libusb-1.0-0-dev).
            Lib::new("libusb", "", "", System)
            .libs(&["usb-1.0"]).libs_win(&["libusb-1.0"]).inc(&["/usr/include/libusb-1.0"])
            .apt(&["libusb-1.0-0-dev"])
            .win_sdk("LIBUSB_DIR", "Download the Windows binaries from https://github.com/libusb/libusb/releases (libusb-1.0.x.7z), extract and set LIBUSB_DIR to that folder")
            .win_inc(&["include/libusb-1.0"]),
            Lib::new("hidapi", &gh("libusb/hidapi"), "hidapi-0.14.0", Cmake)
            .opts(&["-DHIDAPI_BUILD_HIDTEST=OFF"])
            .libs(&["hidapi-hidraw"]).libs_win(&["hidapi"]).inc(&["include/hidapi"])
            .sys_linux(&["udev"]).sys_win(&["setupapi"])
            .apt(&["libudev-dev"]),
            Lib::new("serial", &gh("wjwwood/serial"), "1.2.1", Sources)
            .srcs(&["src/*.cc", "src/impl/*.cc", "src/impl/list_ports/*.cc"]).inc(&["include"])
            .sys_linux(&["pthread"]).sys_win(&["setupapi"]),
            Lib::new("oatpp", &gh("oatpp/oatpp"), "1.3.0-latest", Cmake)
            .opts(&["-DOATPP_BUILD_TESTS=OFF"])
            .libs(&["oatpp"]).inc(&["include/oatpp-1.3.0/oatpp"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32"]),

            // ---------- Dados / compressão / serialização ----------
            // sqlite3: lib do sistema (sudo apt install libsqlite3-dev).
            Lib::new("sqlite3", "", "", System).libs(&["sqlite3"]).sys_linux(&["pthread", "dl", "m"])
            .apt(&["libsqlite3-dev"]),
            // sqlite3 no Windows: o rsfmake troca `sqlite3` por esta entrada (amalgamation compilada junto com o projeto).
            Lib::new("sqlite3_amalgamation", &gh("azadkuh/sqlite-amalgamation"), "master", Sources)
            .srcs(&["sqlite3.c"]).inc(&["."]),
            Lib::new("zstd", &gh("facebook/zstd"), "v1.5.6", Cmake).subdir("build/cmake")
            .opts(&["-DZSTD_BUILD_PROGRAMS=OFF", "-DZSTD_BUILD_TESTS=OFF", "-DZSTD_BUILD_SHARED=OFF",
                  "-DZSTD_BUILD_STATIC=ON"])
            .libs(&["zstd"]).libs_win(&["zstd_static"]).inc(&["include"]),
            Lib::new("lz4", &gh("lz4/lz4"), "v1.10.0", Cmake).subdir("build/cmake")
            .opts(&["-DLZ4_BUILD_CLI=OFF", "-DLZ4_BUILD_LEGACY_LZ4C=OFF", "-DBUILD_STATIC_LIBS=ON"])
            .libs(&["lz4"]).libs_win(&["lz4_static"]).inc(&["include"]),
            Lib::new("flatbuffers", &gh("google/flatbuffers"), "v24.3.25", Cmake)
            .opts(&["-DFLATBUFFERS_BUILD_TESTS=OFF", "-DFLATBUFFERS_BUILD_FLATHASH=OFF",
                  "-DFLATBUFFERS_INSTALL=ON"])
            .libs(&["flatbuffers"]).inc(&["include"]),
            Lib::new("cereal", &gh("USCiLab/cereal"), "v1.3.2", Header).inc(&["include"]),
            Lib::new("simdjson", &gh("simdjson/simdjson"), "v3.10.1", Sources)
            .srcs(&["singleheader/simdjson.cpp"]).inc(&["singleheader"]),
            Lib::new("inih", &gh("benhoyt/inih"), "r58", Sources)
            .srcs(&["ini.c", "cpp/INIReader.cpp"]).inc(&[".", "cpp"]),
            Lib::new("csv_parser", &gh("vincentlaucsb/csv-parser"), "2.3.0", Header)
            .inc(&["single_include"]).sys_linux(&["pthread"]),
            Lib::new("xlnt", &gh("tfussell/xlnt"), "v1.5.0", Cmake)
            .opts(&["-DSTATIC=ON", "-DTESTS=OFF", "-DSAMPLES=OFF", "-DBENCHMARKS=OFF"])
            .libs(&["xlnt"]).inc(&["include"]).defs(&["XLNT_STATIC"]),
            Lib::new("hiredis", &gh("redis/hiredis"), "v1.2.0", Cmake)
            .opts(&["-DDISABLE_TESTS=ON", "-DENABLE_SSL=OFF"])
            .libs(&["hiredis"]).libs_win(&["hiredis_static"]).inc(&["include"])
            .sys_win(&["ws2_32", "crypt32"]),
            // libpqxx: precisa do libpq (sudo apt install libpq-dev).
            Lib::new("libpqxx", &gh("jtv/libpqxx"), "7.9.2", Cmake)
            .opts(&["-DBUILD_TEST=OFF", "-DBUILD_DOC=OFF", "-DSKIP_BUILD_TEST=ON"])
            .libs(&["pqxx"]).inc(&["include"]).sys_linux(&["pq"])
            .apt(&["libpq-dev"]).sys_inc(&["/usr/include/postgresql"])
            .sys_win(&["libpq", "ws2_32"])
            .win_sdk("PGSQL_DIR,PostgreSQL_ROOT", "Install PostgreSQL for Windows (or the EDB binaries zip) and set PGSQL_DIR to its folder (include/ and lib/libpq.lib)")
            .win_zip(&["https://get.enterprisedb.com/postgresql/postgresql-16.4-1-windows-x64-binaries.zip",
                       "https://get.enterprisedb.com/postgresql/postgresql-15.8-1-windows-x64-binaries.zip",
                       "https://get.enterprisedb.com/postgresql/postgresql-14.13-1-windows-x64-binaries.zip"]),
            Lib::new("leveldb", &gh("google/leveldb"), "1.23", Cmake)
            .opts(&["-DLEVELDB_BUILD_TESTS=OFF", "-DLEVELDB_BUILD_BENCHMARKS=OFF", "-DLEVELDB_INSTALL=ON"])
            .libs(&["leveldb"]).inc(&["include"]).sys_linux(&["pthread"]),
            Lib::new("libpng", &gh("pnggroup/libpng"), "v1.6.44", Cmake)
            .opts(&["-DPNG_SHARED=OFF", "-DPNG_TESTS=OFF", "-DPNG_TOOLS=OFF", "-DPNG_FRAMEWORK=OFF"])
            .libs(&["png16"]).libs_win(&["libpng16_static"]).inc(&["include"])
            .deps(&["zlib"]).sys_linux(&["m"]),
            // libjpeg-turbo sem SIMD (evita exigir nasm)
            Lib::new("libjpeg_turbo", &gh("libjpeg-turbo/libjpeg-turbo"), "3.0.4", Cmake)
            .opts(&["-DENABLE_SHARED=OFF", "-DENABLE_STATIC=ON", "-DWITH_SIMD=OFF", "-DWITH_TURBOJPEG=ON"])
            .libs(&["turbojpeg", "jpeg"]).libs_win(&["turbojpeg-static", "jpeg-static"]).inc(&["include"]),
            Lib::new("libwebp", &gh("webmproject/libwebp"), "v1.4.0", Cmake)
            .opts(&["-DWEBP_BUILD_ANIM_UTILS=OFF", "-DWEBP_BUILD_CWEBP=OFF", "-DWEBP_BUILD_DWEBP=OFF",
                  "-DWEBP_BUILD_GIF2WEBP=OFF", "-DWEBP_BUILD_IMG2WEBP=OFF", "-DWEBP_BUILD_VWEBP=OFF",
                  "-DWEBP_BUILD_WEBPINFO=OFF", "-DWEBP_BUILD_WEBPMUX=OFF", "-DWEBP_BUILD_EXTRAS=OFF"])
            .libs(&["webpdemux", "webp", "sharpyuv"])
            .libs_win(&["libwebpdemux", "libwebp", "libsharpyuv"])
            .inc(&["include"]).sys_linux(&["m", "pthread"]),
            Lib::new("opencv", &gh("opencv/opencv"), "4.10.0", Cmake).shared()
            .opts(&["-DBUILD_TESTS=OFF", "-DBUILD_PERF_TESTS=OFF", "-DBUILD_EXAMPLES=OFF",
                  "-DBUILD_DOCS=OFF", "-DBUILD_opencv_apps=OFF", "-DBUILD_opencv_python3=OFF",
                  "-DBUILD_JAVA=OFF", "-DBUILD_opencv_world=ON", "-DOPENCV_GENERATE_PKGCONFIG=OFF"])
            .libs(&["opencv_world"]).libs_win(&["opencv_world4100"]).inc(&["include/opencv4"]),

            // ---------- Texto / regex / parsing ----------
            Lib::new("pcre2", &gh("PCRE2Project/pcre2"), "pcre2-10.44", Cmake)
            .opts(&["-DPCRE2_BUILD_PCRE2GREP=OFF", "-DPCRE2_BUILD_TESTS=OFF", "-DPCRE2_STATIC_PIC=ON",
                  "-DPCRE2_BUILD_PCRE2_8=ON"])
            .libs(&["pcre2-8"]).libs_win(&["pcre2-8-static"]).inc(&["include"])
            .defs(&["PCRE2_STATIC", "PCRE2_CODE_UNIT_WIDTH=8"]),
            Lib::new("utfcpp", &gh("nemtrif/utfcpp"), "v4.0.5", Header).inc(&["source"]),
            Lib::new("pegtl", &gh("taocpp/PEGTL"), "3.2.7", Header).inc(&["include"]),
            Lib::new("date", &gh("HowardHinnant/date"), "v3.0.3", Header).inc(&["include"]),
            Lib::new("exprtk", &gh("ArashPartow/exprtk"), "master", Header).inc(&["."]),
            Lib::new("cxxopts", &gh("jarro2783/cxxopts"), "v3.2.0", Header).inc(&["include"]),
            Lib::new("indicators", &gh("p-ranav/indicators"), "v2.3", Header).inc(&["include"]),
            Lib::new("ftxui", &gh("ArthurSonzogni/FTXUI"), "v5.0.0", Cmake)
            .opts(&["-DFTXUI_BUILD_EXAMPLES=OFF", "-DFTXUI_BUILD_DOCS=OFF", "-DFTXUI_BUILD_TESTS=OFF",
                  "-DFTXUI_ENABLE_INSTALL=ON"])
            .libs(&["ftxui-component", "ftxui-dom", "ftxui-screen"]).inc(&["include"])
            .sys_linux(&["pthread"]),
            Lib::new("easyloggingpp", &gh("abumq/easyloggingpp"), "v9.97.1", Header).inc(&["src"]),
            Lib::new("cpptrace", &gh("jeremy-rifkin/cpptrace"), "v0.7.2", Cmake).shared()
            .opts(&["-DCPPTRACE_BUILD_TESTING=OFF"])
            .libs(&["cpptrace"]).inc(&["include"]),

            // ---------- Utilitários C++ / estruturas ----------
            Lib::new("taskflow", &gh("taskflow/taskflow"), "v3.8.0", Header).inc(&["."])
            .sys_linux(&["pthread"]),
            Lib::new("parallel_hashmap", &gh("greg7mdp/parallel-hashmap"), "v1.3.12", Header).inc(&["."]),
            Lib::new("robin_hood", &gh("martinus/robin-hood-hashing"), "3.11.5", Header).inc(&["src/include"]),
            Lib::new("plf_colony", &gh("mattreecebentley/plf_colony"), "master", Header).inc(&["."]),
            Lib::new("boost_di", &gh("boost-ext/di"), "v1.3.0", Header).inc(&["include"]),
            Lib::new("gsl", &gh("microsoft/GSL"), "v4.0.0", Header).inc(&["include"]),
            Lib::new("xtl", &gh("xtensor-stack/xtl"), "0.7.7", Header).inc(&["include"]),
            Lib::new("xtensor", &gh("xtensor-stack/xtensor"), "0.25.0", Header).inc(&["include"])
            .deps(&["xtl"]),
            Lib::new("pcg_cpp", &gh("imneme/pcg-cpp"), "master", Header).inc(&["include"]),
            Lib::new("xxhash", &gh("Cyan4973/xxHash"), "v0.8.2", Sources)
            .srcs(&["xxhash.c"]).inc(&["."]),
            Lib::new("subprocess", &gh("arun11299/cpp-subprocess"), "v2.0", Header).inc(&["."]),
            Lib::new("dylib", &gh("martin-olivier/dylib"), "v2.2.1", Header).inc(&["include"])
            .sys_linux(&["dl"]),
            Lib::new("chaiscript", &gh("ChaiScript/ChaiScript"), "v6.1.0", Header).inc(&["include"])
            .sys_linux(&["dl", "pthread"]),
            // pybind11: precisa dos headers/libs do Python no include path do projeto (cxxflags/libs).
            Lib::new("pybind11", &gh("pybind/pybind11"), "v2.13.6", Header).inc(&["include"]),
            Lib::new("benchmark", &gh("google/benchmark"), "v1.9.0", Cmake)
            .opts(&["-DBENCHMARK_ENABLE_TESTING=OFF", "-DBENCHMARK_ENABLE_GTEST_TESTS=OFF",
                  "-DBENCHMARK_ENABLE_INSTALL=ON"])
            .libs(&["benchmark_main", "benchmark"]).inc(&["include"])
            .sys_linux(&["pthread"]).sys_win(&["shlwapi"]),
            Lib::new("mimalloc", &gh("microsoft/mimalloc"), "v2.1.7", Cmake)
            .opts(&["-DMI_BUILD_TESTS=OFF", "-DMI_BUILD_SHARED=OFF", "-DMI_BUILD_OBJECT=OFF"])
            .libs(&["mimalloc"]).libs_win(&["mimalloc-static"]).inc(&["include"])
            .sys_linux(&["pthread", "rt"]).sys_win(&["psapi", "advapi32", "bcrypt"]),
            Lib::new("rpmalloc", &gh("mjansson/rpmalloc"), "1.4.5", Sources)
            .srcs(&["rpmalloc/rpmalloc.c"]).inc(&["rpmalloc"]).sys_linux(&["pthread"]),

            // ---------- Geometria / jogos / física / navegação ----------
            Lib::new("earcut", &gh("mapbox/earcut.hpp"), "v2.2.4", Header).inc(&["include"]),
            Lib::new("clipper2", &gh("AngusJohnson/Clipper2"), "Clipper2_1.4.0", Cmake).subdir("CPP")
            .opts(&["-DCLIPPER2_UTILS=OFF", "-DCLIPPER2_EXAMPLES=OFF", "-DCLIPPER2_TESTS=OFF"])
            .libs(&["Clipper2"]).inc(&["include"]),
            Lib::new("fastnoise2", &gh("Auburn/FastNoise2"), "v0.10.0-alpha", Cmake)
            .opts(&["-DFASTNOISE2_NOISETOOL=OFF", "-DFASTNOISE2_TESTS=OFF"])
            .libs(&["FastNoise"]).inc(&["include"]),
            Lib::new("tiled_tmxlite", &gh("fallahn/tmxlite"), "v1.4.4", Cmake).subdir("tmxlite")
            .opts(&["-DTMXLITE_STATIC_LIB=ON"])
            .libs(&["tmxlite-s"]).inc(&["include"]),
            Lib::new("recastnavigation", &gh("recastnavigation/recastnavigation"), "v1.6.0", Cmake)
            .opts(&["-DRECASTNAVIGATION_DEMO=OFF", "-DRECASTNAVIGATION_TESTS=OFF",
                  "-DRECASTNAVIGATION_EXAMPLES=OFF"])
            .libs(&["DebugUtils", "DetourCrowd", "DetourTileCache", "Detour", "Recast"])
            .inc(&["include/recastnavigation"]),
            Lib::new("flecs", &gh("SanderMertens/flecs"), "v4.0.3", Cmake)
            .opts(&["-DFLECS_STATIC=ON", "-DFLECS_SHARED=OFF", "-DFLECS_TESTS=OFF"])
            .libs(&["flecs"]).libs_win(&["flecs_static"]).inc(&["include"]).defs(&["flecs_STATIC"])
            .sys_linux(&["pthread"]).sys_win(&["ws2_32"]),
            // liquidfun: fork do Box2D com partículas; conflita com `box2d` se usados juntos.
            Lib::new("liquidfun", &gh("google/liquidfun"), "master", Sources)
            .srcs(&["liquidfun/Box2D/Box2D/Collision/*.cpp", "liquidfun/Box2D/Box2D/Collision/Shapes/*.cpp",
                  "liquidfun/Box2D/Box2D/Common/*.cpp", "liquidfun/Box2D/Box2D/Dynamics/*.cpp",
                  "liquidfun/Box2D/Box2D/Dynamics/Contacts/*.cpp", "liquidfun/Box2D/Box2D/Dynamics/Joints/*.cpp",
                  "liquidfun/Box2D/Box2D/Particle/*.cpp", "liquidfun/Box2D/Box2D/Rope/*.cpp"])
            .inc(&["liquidfun/Box2D"]),

            // ---------- IA / ML ----------
            // llama.cpp / ggml / whisper.cpp mudam o nome das libs entre versões; usamos master. Não misture os três (ggml duplicado).
            Lib::new("llama_cpp", &gh("ggerganov/llama.cpp"), "master", Cmake)
            .opts(&["-DLLAMA_BUILD_TESTS=OFF", "-DLLAMA_BUILD_EXAMPLES=OFF", "-DLLAMA_BUILD_SERVER=OFF",
                  "-DLLAMA_CURL=OFF", "-DGGML_OPENMP=OFF"])
            .libs(&["llama", "ggml", "ggml-cpu", "ggml-base"]).inc(&["include"])
            .sys_linux(&["pthread", "m", "dl"]),
            Lib::new("ggml", &gh("ggerganov/ggml"), "master", Cmake)
            .opts(&["-DGGML_BUILD_TESTS=OFF", "-DGGML_BUILD_EXAMPLES=OFF", "-DGGML_OPENMP=OFF"])
            .libs(&["ggml", "ggml-cpu", "ggml-base"]).inc(&["include"])
            .sys_linux(&["pthread", "m", "dl"]),
            Lib::new("whisper_cpp", &gh("ggerganov/whisper.cpp"), "master", Cmake)
            .opts(&["-DWHISPER_BUILD_TESTS=OFF", "-DWHISPER_BUILD_EXAMPLES=OFF", "-DWHISPER_BUILD_SERVER=OFF",
                  "-DGGML_OPENMP=OFF"])
            .libs(&["whisper", "ggml", "ggml-cpu", "ggml-base"]).inc(&["include"])
            .sys_linux(&["pthread", "m", "dl"]),
            // onnxruntime: baixe o release pré-compilado (github.com/microsoft/onnxruntime/releases) e extraia em redist/_sdk/onnxruntime
            Lib::new("onnxruntime", "", "", Prebuilt)
            .prebuilt("ONNXRUNTIME_SDK", "onnxruntime", "lib", "lib",
                      "Download an ONNX Runtime release (github.com/microsoft/onnxruntime/releases) and extract it into redist/_sdk/onnxruntime (or set ONNXRUNTIME_SDK)")
            .inc(&["include"]).libs(&["onnxruntime"]),

            // ---------- Complementares ----------
            // openssl: lib do sistema (sudo apt install libssl-dev). No Windows instale o OpenSSL e ajuste cxxflags/libs.
            Lib::new("openssl", "", "", System)
            .libs(&["ssl", "crypto"]).libs_win(&["libssl", "libcrypto"])
            .sys_linux(&["dl", "pthread"]).sys_win(&["ws2_32", "crypt32"])
            .apt(&["libssl-dev"])
            .win_sdk("OPENSSL_ROOT_DIR,OPENSSL_DIR", "Install OpenSSL for Windows (e.g. slproweb.com/products/Win32OpenSSL.html or `vcpkg install openssl`) and set OPENSSL_ROOT_DIR to its folder (include/ and lib/)"),
            Lib::new("portable_file_dialogs", &gh("samhocevar/portable-file-dialogs"), "0.1.0", Header)
            .inc(&["."]).sys_win(&["ole32", "shell32", "uuid"]),
      ];

      list.into_iter().map(|l| (l.name.clone(), l)).collect()
}