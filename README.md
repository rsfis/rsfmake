# rsfmake
### rsfmake is a opensource package manager for C++, inspired on Cargo.

Commands:
| Comands | Description |
|---|---|
| `rsfmake new <name>` | Creates a new project in folder `<name>` |
| `rsfmake init` | Creates `rsfmake.toml` and `src/main.cpp` in the current directory |
| `rsfmake build [flags]` | Downloads and compiles the dependencies and compiles the project |
| `rsfmake run [flags] [-- args]` | Compiles and executes the project |
| `rsfmake deps [flags]` | Only download and compiles the dependencies |
| `rsfmake clean [--all]` | Removes `bin/` (with `--all` also removes `redist/`) |
| `rsfmake list` | Lists the available libraries |
| `rsfmake --help` | Help |
| `rsfmake --version` | Shows the version |

## Flags

For build, run, deps

| Flag | Description | Values | Default |
|---|---|---|---|
| `-r`, `--release` | Compiles in release mode | (without it: ) | debug |
| `--target <alvo>` | Target platform | `linux`, `windows` (aliases: `win`, `win32`) | host current running platform |
| `--arch <arq>` | Arquitetura de destino | `x64` (`x86_64`, `amd64`), `arm64` (`aarch64`) | host current running platform |

Other comands

| Command | Flag | Description |
|---|---|---|
| `clean` | `--all` | Removes also `redist/` (downloaded and compiled dependencies) |
| `run` | `-- <args>` | Everything after `--` is passed for the program |

run -- <args> example:

```bash
rsfmake run -r -- --fullscreen    # -r goes for the compiler; --fullscreen is your program's flag to deal
rsfmake run --fullscreen          # error
```

# Examples

```bash
rsfmake new mygame             # creates the project
cd mygame
rsfmake run                    # runs on debug mode on the current host platform
rsfmake run --release          # runs on release mode on the current host platform
rsfmake run -- --fullscreen    # passes the fullscreen argument for the program
rsfmake build -r               # only compiles for release
rsfmake deps                   # only prepares the dependencies
rsfmake build --target windows # cross-compile for Windows (from Linux, via MinGW)
rsfmake build --arch arm64     # cross-compile for ARM64 (from Linux x64)
rsfmake list                   # lists the available libs
rsfmake clean                  # cleans bin/
rsfmake clean --all            # cleans bin/ e redist/
```

# How to install
### On Linux Ubuntu and Debian:
```bash
sudo apt update
sudo apt install -y build-essential cmake ninja-build git pkg-config python3
sudo apt install -y g++-mingw-w64-x86-64    # gerar .exe: rsfmake build --target windows
sudo apt install -y g++-aarch64-linux-gnu   # ARM64:      rsfmake build --arch arm64
```

### On Linux Arch:
```bash
sudo pacman -S --needed base-devel cmake ninja git python
```

### On Linux Fedora:
```bash
sudo dnf install -y gcc-c++ cmake ninja-build git pkgconf python3
```

### On Windows:
```bash
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --id Kitware.CMake
winget install --id Ninja-build.Ninja
winget install --id Git.Git
winget install --id Python.Python.3.12
winget install --id KhronosGroup.VulkanSDK
```

# Available Libs:
Use the library name in the `[dependencies]` section of `rsfmake.toml`:

```toml
[dependencies]
glfw = "*"        # default version from the registry
glm = "1.0.0"     # overrides the registry tag
```

To see the list directly in the terminal: `rsfmake list`.

**Kinds:** `cmake` (clones and builds) · `header` (clone only) · `sources` (compiled together with your project) · `system` (system library, nothing is downloaded) · `sdk` (proprietary SDK downloaded by hand)

---

## System
- `opengl` (system)
- `vulkan` (system, uses `VULKAN_SDK` if set; on Windows requires the LunarG SDK)
- `openssl` (system, `libssl-dev`; on Windows uses `OPENSSL_ROOT_DIR`)
- `sqlite3` (system, `libsqlite3-dev`; on Windows swapped for `sqlite3_amalgamation`)
- `mysqlpp` (system) — MySQL++, `sudo apt install libmysql++-dev`; on Windows uses `MYSQLPP_DIR`
- `gpsd` (system) — libgps, Linux only, `sudo apt install libgps-dev`
- `ffmpeg` (system, `libav*-dev`; on Windows uses `FFMPEG_DIR` or auto-download)
- `libusb` (system, `libusb-1.0-0-dev`; on Windows uses `LIBUSB_DIR`)

## Windowing, input and multimedia
- `glfw` (cmake, 3.4)
- `sdl2` (cmake, release-2.30.9)
- `sdl3` (cmake, release-3.2.0)
- `sdl2_image` (cmake, release-2.8.2)
- `sdl2_ttf` (cmake, release-2.22.0)
- `sdl2_mixer` (cmake, release-2.8.0)
- `sfml` (cmake, 3.0.0)
- `hidapi` (cmake, hidapi-0.14.0)
- `serial` (sources, 1.2.1) — serial port access
- `portable_file_dialogs` (header, 0.1.0)

## Terminal / TUI
- `ncurses` (system, 6.5) — uses `ncursesw` on Linux; replaced by `pdcurses` on Windows
- `ncursesw` (system) — explicit alias of `ncurses`
- `pdcurses` (sources, v4.4.0) — PDCursesMod, Windows only
- `ftxui` (cmake, v5.0.0)
- `indicators` (header, v2.3)

## OpenGL / Vulkan / rendering
- `glad` (cmake, v0.1.36) — requires `python3`; defines `GLFW_INCLUDE_NONE`
- `vulkan_headers` (header, v1.3.296)
- `vma` (header, v3.1.0)
- `volk` (sources, vulkan-sdk-1.3.296.0)
- `meshoptimizer` (cmake, v0.22)
- `bgfx` (cmake, master) — via bgfx.cmake (bx, bimg, bgfx)
- `spirv_cross` (cmake, vulkan-sdk-1.3.296.0)
- `ktx` (cmake, v4.3.2)

## Graphics, games and engines
- `raylib` (cmake, 5.5)
- `ogre` (cmake, v14.3.2) — heavy build
- `entt` (header, v3.14.0)
- `flecs` (cmake, v4.0.3)
- `enet` (cmake, v1.3.18)
- `tracy` (sources, v0.11.1)
- `optick` (sources, master)
- `tiled_tmxlite` (cmake, v1.4.4)
- `recastnavigation` (cmake, v1.6.0)
- `fastnoise2` (cmake, v0.10.0-alpha)
- `earcut` (header, v2.2.4)
- `clipper2` (cmake, Clipper2_1.4.0)

## Physics
- `bullet3` (cmake, 3.25)
- `joltphysics` (cmake, v5.2.0)
- `reactphysics3d` (cmake, v0.10.2)
- `box2d` (cmake, v2.4.2)
- `liquidfun` (sources, master) — Box2D fork with particles; conflicts with `box2d`

## Math and scientific
- `glm` (header, 1.0.1)
- `eigen` (header, 3.4.0)
- `cglm` (header, v0.9.4)
- `armadillo` (header, 14.0.3) — needs `libopenblas-dev`; on Windows uses `OPENBLAS_DIR` or auto-download
- `xtl` (header, 0.7.7)
- `xtensor` (header, 0.25.0, depends on `xtl`)
- `exprtk` (header, master)
- `pcg_cpp` (header, master)

## 3D models and images
- `assimp` (cmake, v5.4.3)
- `tinygltf` (header, v2.9.3)
- `cgltf` (header, v1.14)
- `tinyobjloader` (header, v2.0.0rc13)
- `stb` (header, master)
- `stb_image` (header, master) — alias of `stb`
- `freetype` (cmake, VER-2-13-3)
- `harfbuzz` (cmake, 10.0.1, depends on `freetype`)
- `libpng` (cmake, v1.6.44, depends on `zlib`)
- `libjpeg_turbo` (cmake, 3.0.4)
- `libwebp` (cmake, v1.4.0)
- `opencv` (cmake, 4.10.0) — `opencv_world`, heavy build

## Data and formats
- `nlohmann_json` (header, v3.11.3)
- `rapidjson` (header, v1.1.0)
- `simdjson` (sources, v3.10.1)
- `jsoncpp` (cmake, 1.9.6) — mainly a Windows dependency of `drogon`
- `pugixml` (cmake, v1.14)
- `tinyxml2` (cmake, 10.0.0)
- `tinyxml` (cmake, 10.0.0) — alias of `tinyxml2`
- `tomlplusplus` (header, v3.4.0)
- `toml` (header, v3.4.0) — alias of `tomlplusplus`
- `yaml_cpp` (cmake, 0.8.0)
- `inih` (sources, r58)
- `csv_parser` (header, 2.3.0)
- `xlnt` (cmake, v1.5.0) — xlsx
- `flatbuffers` (cmake, v24.3.25)
- `cereal` (header, v1.3.2)

## Compression
- `zlib` (cmake, v1.3.1)
- `zstd` (cmake, v1.5.6)
- `lz4` (cmake, v1.10.0)

## Databases
- `sqlite3` (system) — see System
- `sqlite3_amalgamation` (sources, master) — used on Windows in place of `sqlite3`
- `sqlitecpp` (cmake, 3.3.2)
- `mysqlpp` (system) — see System
- `qtl` (header, master)
- `libpqxx` (cmake, 7.9.2) — needs `libpq-dev`; on Windows uses `PGSQL_DIR` or auto-download
- `hiredis` (cmake, v1.2.0)
- `leveldb` (cmake, 1.23)

## Audio
- `openal` (cmake, 1.23.1)
- `miniaudio` (header, 0.11.21)
- `dr_libs` (header, master)
- `soloud` (sources, RELEASE_20200207) — miniaudio backend
- `libsndfile` (cmake, 1.2.2)
- `rtaudio` (cmake, 6.0.1)
- `libogg` (cmake, v1.3.5)
- `libvorbis` (cmake, v1.3.7, depends on `libogg`)
- `lilbvorbis` (cmake, v1.3.7) — alias of `libvorbis`
- `opus` (cmake, v1.5.2)
- `fmod` (sdk) — download into redist/_sdk/fmod
- `fmodstudio` (sdk, depends on `fmod`)
- `steamworks` (sdk) — download into redist/_sdk/steamworks

## User interface (UI)
- `imgui` (sources, v1.91.5)
- `imgui_glfw_gl3` (sources) — imgui + GLFW + OpenGL3
- `imgui_glfw_vulkan` (sources) — imgui + GLFW + Vulkan
- `imgui_sdl2_gl3` (sources) — imgui + SDL2 + OpenGL3
- `implot` (sources, v0.16)
- `imgui_node_editor` (sources, master)
- `imguizmo` (sources, master)
- `nanogui` (cmake, master)
- `rmlui` (cmake, 6.0, depends on `freetype`)

## Scripting
- `lua` (sources, v5.4.7)
- `sol2` (header, v3.3.0, depends on `lua`)
- `chaiscript` (header, v6.1.0)
- `pybind11` (header, v2.13.6) — needs Python headers/libs in the project flags

## AI / ML
- `llama_cpp` (cmake, master)
- `ggml` (cmake, master)
- `whisper_cpp` (cmake, master)
- `onnxruntime` (sdk) — download into redist/_sdk/onnxruntime
- Note: do not mix `llama_cpp`, `ggml` and `whisper_cpp` (duplicate ggml)

## Networking, web and concurrency
- `asio` (header, asio-1-30-2)
- `httplib` (header, v0.18.1) — cpp-httplib
- `curl` (cmake, curl-8_10_1) — needs `libssl-dev` on Linux
- `cpr` (cmake, 1.11.0, depends on `curl`)
- `websocketpp` (header, 0.8.2, depends on `asio`)
- `ixwebsocket` (cmake, v11.4.5)
- `crow` (header, v1.2.0, depends on `asio`)
- `drogon` (cmake, v1.9.7) — needs `libjsoncpp-dev` and `uuid-dev`; TLS disabled
- `oatpp` (cmake, 1.3.0-latest)
- `eventpp` (header, v0.1.3)
- `concurrentqueue` (header, v1.0.4)
- `taskflow` (header, v3.8.0)

## C++ utilities
- `fmt` (cmake, 11.0.2)
- `spdlog` (cmake, v1.14.1)
- `easyloggingpp` (header, v9.97.1)
- `cpptrace` (cmake, v0.7.2)
- `cli11` (header, v2.4.2)
- `cxxopts` (header, v3.2.0)
- `magic_enum` (header, v0.9.6)
- `range_v3` (header, 0.12.0)
- `date` (header, v3.0.3)
- `gsl` (header, v4.0.0)
- `boost_di` (header, v1.3.0)
- `unordered_dense` (header, v4.4.0)
- `parallel_hashmap` (header, v1.3.12)
- `robin_hood` (header, 3.11.5)
- `plf_colony` (header, master)
- `xxhash` (sources, v0.8.2)
- `subprocess` (header, v2.0)
- `dylib` (header, v2.2.1)
- `mimalloc` (cmake, v2.1.7)
- `rpmalloc` (sources, 1.4.5)

## Text / parsing
- `pcre2` (cmake, pcre2-10.44)
- `utfcpp` (header, v4.0.5)
- `pegtl` (header, 3.2.7)

## Testing and benchmarking
- `catch2` (cmake, v3.7.1)
- `doctest` (header, v2.4.11)
- `googletest` (cmake, v1.15.2)
- `benchmark` (cmake, v1.9.0)

---

# Adding your own library

Any git repository can become a dependency, directly in `rsfmake.toml`, using the table format:

```toml
[dependencies]
mylib = { git = "https://github.com/x/y.git", tag = "v1.0", libs = ["y"], inc = ["include"] }
```

## Fields

- `git` — repository URL (required, except for `system` and `prebuilt`)
- `tag` — tag, branch or release (default: `master`)
- `kind` — `cmake`, `header`, `sources`, `system` or `prebuilt`
- `subdir` — repo subfolder that contains the `CMakeLists.txt` (e.g. `"Build"`)
- `cmake` — extra CMake options (e.g. `["-DY_TESTS=OFF"]`)
- `libs` — library names to link (e.g. `["y"]`)
- `libs_win` — library names on Windows, if different
- `sys` — system libraries on Linux (e.g. `["pthread", "dl", "m"]`)
- `sys_win` — system libraries on Windows (e.g. `["ws2_32"]`)
- `inc` — include folders (see the rule below)
- `defines` — `-D` macros added to your project
- `deps` — other libraries this one depends on (from the registry or from your toml)
- `sources` — `.c`/`.cpp` files from the repo compiled together with your project
- `shared` — `true` if the library is dynamic (`.so`/`.dll`)
- `sdk_env` — environment variable holding the SDK path (`prebuilt` only)
- `sdk_dir` — folder under `redist/_sdk/` (default: the library name)
- `libdir` — library folder inside the SDK; accepts `{arch}` (`prebuilt` only)
- `libdir_win` — same, for Windows

## How `inc` is interpreted

- `cmake` — relative to the install prefix (`redist/<plat>/<arch>`)
- `header` and `sources` — relative to the downloaded source code
- `system` — absolute path

## How `kind` is chosen if you don't set it

- If it has `libs` or `cmake` → `cmake`
- Otherwise, if it has `sources` → `sources`
- Otherwise → `header`

## Examples

**Library built with CMake:**
```toml
mylib = { git = "https://github.com/x/y.git", tag = "v1.0", cmake = ["-DY_TESTS=OFF", "-DY_EXAMPLES=OFF"], libs = ["y"], inc = ["include"] }
```

**Header-only:**
```toml
otherlib = { git = "https://github.com/x/z.git", tag = "v2.3.0", kind = "header", inc = ["include"] }
```

**Source code compiled together with your project:**
```toml
miniparser = { git = "https://github.com/x/mp.git", tag = "main", kind = "sources", sources = ["src/mp.c"], inc = ["include"] }
```

**With a dependency on another library and system libs:**
```toml
netlib = { git = "https://github.com/x/net.git", tag = "v1.2", libs = ["net"], inc = ["include"], deps = ["zlib"], sys = ["pthread"], sys_win = ["ws2_32"] }
```

**CMakeLists in a subfolder, dynamic library and macro:**
```toml
engine = { git = "https://github.com/x/engine.git", tag = "v3.0", subdir = "build", libs = ["engine"], libs_win = ["engine64"], shared = true, defines = ["ENGINE_STATIC"] }
```

**System library:**
```toml
sqlite = { kind = "system", libs = ["sqlite3"] }
```

**Proprietary SDK downloaded by hand:**
```toml
mysdk = { kind = "prebuilt", sdk_env = "MYSDK", sdk_dir = "mysdk", libdir = "lib/{arch}", libdir_win = "lib/{arch}", inc = ["include"], libs = ["mysdk"], libs_win = ["mysdk64"] }
```
Extract the SDK into `redist/_sdk/mysdk` (or set the `MYSDK` variable).

## Tips

- Order in `libs` matters on Linux with static libraries: put the one that depends on the others first (e.g. `["BulletDynamics", "BulletCollision", "LinearMath"]`).
- If you change `tag`, `cmake` or `libs` of an already-built library, run `rsfmake clean --all` to rebuild the cache.
- To test your configuration without building the project, use `rsfmake deps`.

# How `rsfmake.toml` works

`rsfmake.toml` is the manifest of your project. It tells rsfmake what to build and which libraries to download and link. It has two sections:

- `[package]` — describes your project
- `[dependencies]` — lists the libraries it uses

rsfmake looks for the file in the current directory and then in each parent directory, so you can run commands from any subfolder of the project.

## Minimal example

```toml
[package]
name = "mygame"

[dependencies]
glfw = "*"
glm = "*"
```

Everything else has a default value.

---

## `[package]`

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | string | — (required) | Project and output file name |
| `version` | string | `"0.1.0"` | Informational only, not used by the build |
| `type` | string | `"exe"` | `exe`, `static` or `shared` |
| `std` | number | `20` | C++ standard (`17`, `20`, `23`, ...) |
| `sources` | list of strings | `["src/*.cpp", "src/*.c", "src/*.cc"]` | Source file patterns, relative to the project root |
| `include` | list of strings | `["include", "src"]` | Include folders of your own code |
| `defines` | list of strings | `[]` | Preprocessor macros, without `-D` (e.g. `"DEBUG_UI"`) |
| `cxxflags` | list of strings | `[]` | Extra compiler flags (e.g. `"-Wall"`) |
| `libs` | list of strings | `[]` | Extra libraries to link (e.g. `"pthread"`) |
| `assets` | string | none | Folder copied next to the executable after the build |

### Details

- **`type`** controls what is produced: `exe` builds an executable, `static` a static library, `shared` a dynamic library. Any other value is an error.
- **`sources`** patterns are searched recursively, so `src/*.cpp` also finds files in subfolders such as `src/render/`.
- **`include`** folders are added to the compiler include path for your own headers. Library headers are added automatically from `[dependencies]`.
- **`defines`** are passed as `-D` macros, together with the macros that some libraries add on their own.
- **`libs`** is appended at the end of the link line, after the libraries from `[dependencies]`.
- **`assets`** is copied to the output folder, keeping its name. For `assets = "assets"`, the folder ends up at `bin/<platform>+<config>/assets`.

### Full `[package]` example

```toml
[package]
name = "mygame"
version = "0.2.0"
type = "exe"
std = 20
sources = ["src/*.cpp", "src/*.c", "third_party/*.c"]
include = ["include", "src", "third_party"]
defines = ["GAME_VERSION=2"]
cxxflags = ["-Wall", "-Wextra"]
libs = ["pthread"]
assets = "assets"
```

---

## `[dependencies]`

Each line is one library. There are three ways to write it.

### 1. Registry library, default version

```toml
glfw = "*"
```

`"*"` (or an empty string) uses the tag defined in rsfmake's built-in registry. The list is in `LIBS.md`, or run `rsfmake list`.

### 2. Registry library, specific version

```toml
glm = "0.9.9.8"
```

Any other string replaces the registry tag. It is used as a git tag or branch when cloning, so it must exist in the library repository.

### 3. Custom library

```toml
mylib = { git = "https://github.com/x/y.git", tag = "v1.0", libs = ["y"], inc = ["include"] }
```

Any git repository can be used. All fields are described in `LIBS.md`, in the "Adding your own library" section.

### What happens when you list a library

- **Transitive dependencies are added automatically.** For example, `imgui_glfw_gl3` brings `imgui`, `glfw` and `opengl`. You do not need to list them.
- **Unknown names are an error.** If the name is not in the registry and you did not use the table form with `git`, rsfmake stops with "unknown library".
- **Order in the file does not matter.** Dependencies are processed in alphabetical order, with their own dependencies first. Link order is handled for you.
- **Defines are merged.** If a library needs a macro (such as `SPDLOG_COMPILED_LIB`), it is added to your project automatically.

---

## What rsfmake does with the manifest

1. Reads `rsfmake.toml` and resolves the full dependency list.
2. Clones each library into `redist/_src/` (shallow clone, only the requested tag).
3. Builds `cmake` libraries with CMake and Ninja, and installs them into `redist/<platform>/<arch>/`. A stamp file avoids rebuilding libraries that are already built.
4. Generates a `CMakeLists.txt` in `bin/build_temp/` with your sources, includes, defines, and link libraries.
5. Builds the project into `bin/<platform>+<config>/`, copies the runtime libraries (`.so`/`.dll`) and the `assets` folder next to it.

You never write a `CMakeLists.txt` yourself.

---

## Good to know

- **Typos are silent.** Unknown keys are ignored, so `stdd = 17` will not give an error and the default is used instead. Check spelling if a setting seems to have no effect.
- **Changing a library version or options** after it was built does not always trigger a rebuild. Run `rsfmake clean --all` to rebuild everything.
- **The `[package]` `libs` field** is for libraries you link by name (system libraries, for example). Use `[dependencies]` for everything that rsfmake should download.
- **`rsfmake new` and `rsfmake init`** generate a starter `rsfmake.toml` with `nlohmann_json` and commented examples of other libraries.

---

## Complete example

```toml
[package]
name = "mygame"
version = "0.1.0"
type = "exe"
std = 20
sources = ["src/*.cpp"]
include = ["include", "src"]
defines = ["GAME_DEBUG"]
cxxflags = ["-Wall"]
assets = "assets"

[dependencies]
# libraries from the registry
glfw = "*"
glad = "*"
glm = "*"
nlohmann_json = "*"
imgui_glfw_gl3 = "*"
spdlog = "v1.14.1"   # must match the git tag exactly

# custom library from a git repository
mylib = { git = "https://github.com/x/y.git", tag = "v1.0", libs = ["y"], inc = ["include"] }
```
