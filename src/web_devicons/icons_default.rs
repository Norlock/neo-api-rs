use std::{collections::HashMap, ffi::OsStr, path::Path};

use mlua::prelude::LuaResult;
use mlua::Lua;
use once_cell::sync::Lazy;

use crate::{HLOpts, NeoTheme};

struct DevIconsContainer {
    from_file_name: HashMap<&'static OsStr, DevIcon>,
    from_extension: HashMap<&'static OsStr, DevIcon>,
    from_os: HashMap<&'static OsStr, DevIcon>,
    from_de: HashMap<&'static OsStr, DevIcon>,
    from_wm: HashMap<&'static OsStr, DevIcon>,
}

// https://github.com/nvim-tree/nvim-web-devicons
static DEV_ICONS: Lazy<DevIconsContainer> = Lazy::new(|| {
    let dev_icon_from_filename = HashMap::from_iter([
        (
            OsStr::new("build.gradle"),
            DevIcon {
                icon: "",
                color: "#005f87",
                name: "GradleBuildScript",
            },
        ),
        (
            OsStr::new("settings.gradle"),
            DevIcon {
                icon: "",
                color: "#005f87",

                name: "GradleSettings",
            },
        ),
        (
            OsStr::new(".babelrc"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Babelrc",
            },
        ),
        (
            OsStr::new(".bash_profile"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "BashProfile",
            },
        ),
        (
            OsStr::new(".bashrc"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Bashrc",
            },
        ),
        (
            OsStr::new(".dockerignore"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new(".ds_store"),
            DevIcon {
                icon: "",
                color: "#41535b",

                name: "DsStore",
            },
        ),
        (
            OsStr::new(".editorconfig"),
            DevIcon {
                icon: "",
                color: "#fff2f2",

                name: "EditorConfig",
            },
        ),
        (
            OsStr::new(".env"),
            DevIcon {
                icon: "",
                color: "#faf743",

                name: "Env",
            },
        ),
        (
            OsStr::new(".eslintrc"),
            DevIcon {
                icon: "",
                color: "#4b32c3",

                name: "Eslintrc",
            },
        ),
        (
            OsStr::new(".eslintignore"),
            DevIcon {
                icon: "",
                color: "#4b32c3",

                name: "EslintIgnore",
            },
        ),
        (
            OsStr::new(".gitattributes"),
            DevIcon {
                icon: "",
                color: "#f54d27",

                name: "GitAttributes",
            },
        ),
        (
            OsStr::new(".gitconfig"),
            DevIcon {
                icon: "",
                color: "#f54d27",

                name: "GitConfig",
            },
        ),
        (
            OsStr::new(".gitignore"),
            DevIcon {
                icon: "",
                color: "#f54d27",

                name: "GitIgnore",
            },
        ),
        (
            OsStr::new(".gitlab-ci.yml"),
            DevIcon {
                icon: "",
                color: "#e24329",

                name: "GitlabCI",
            },
        ),
        (
            OsStr::new(".gitmodules"),
            DevIcon {
                icon: "",
                color: "#f54d27",

                name: "GitModules",
            },
        ),
        (
            OsStr::new(".gtkrc-2.0"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "GTK",
            },
        ),
        (
            OsStr::new(".gvimrc"),
            DevIcon {
                icon: "",
                color: "#019833",

                name: "Gvimrc",
            },
        ),
        (
            OsStr::new(".luaurc"),
            DevIcon {
                icon: "",
                color: "#00a2ff",

                name: "Luaurc",
            },
        ),
        (
            OsStr::new(".mailmap"),
            DevIcon {
                icon: "󰊢",
                color: "#41535b",

                name: "Mailmap",
            },
        ),
        (
            OsStr::new(".npmignore"),
            DevIcon {
                icon: "",
                color: "#E8274B",

                name: "NPMIgnore",
            },
        ),
        (
            OsStr::new(".npmrc"),
            DevIcon {
                icon: "",
                color: "#E8274B",

                name: "NPMrc",
            },
        ),
        (
            OsStr::new(".prettierrc"),
            DevIcon {
                icon: "",
                color: "#4285F4",

                name: "PrettierConfig",
            },
        ),
        (
            OsStr::new(".settings.json"),
            DevIcon {
                icon: "",
                color: "#854CC7",

                name: "SettingsJson",
            },
        ),
        (
            OsStr::new(".SRCINFO"),
            DevIcon {
                icon: "󰣇",
                color: "#0f94d2",

                name: "SRCINFO",
            },
        ),
        (
            OsStr::new(".vimrc"),
            DevIcon {
                icon: "",
                color: "#019833",

                name: "Vimrc",
            },
        ),
        (
            OsStr::new(".Xauthority"),
            DevIcon {
                icon: "",
                color: "#e54d18",

                name: "Xauthority",
            },
        ),
        (
            OsStr::new(".xinitrc"),
            DevIcon {
                icon: "",
                color: "#e54d18",

                name: "XInitrc",
            },
        ),
        (
            OsStr::new(".Xresources"),
            DevIcon {
                icon: "",
                color: "#e54d18",

                name: "Xresources",
            },
        ),
        (
            OsStr::new(".xsession"),
            DevIcon {
                icon: "",
                color: "#e54d18",

                name: "Xsession",
            },
        ),
        (
            OsStr::new(".zprofile"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Zshprofile",
            },
        ),
        (
            OsStr::new(".zshenv"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Zshenv",
            },
        ),
        (
            OsStr::new(".zshrc"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Zshrc",
            },
        ),
        (
            OsStr::new("_gvimrc"),
            DevIcon {
                icon: "",
                color: "#019833",

                name: "Gvimrc",
            },
        ),
        (
            OsStr::new("_vimrc"),
            DevIcon {
                icon: "",
                color: "#019833",

                name: "Vimrc",
            },
        ),
        (
            OsStr::new("R"),
            DevIcon {
                icon: "󰟔",
                color: "#2266ba",

                name: "R",
            },
        ),
        (
            OsStr::new("avif"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Avif",
            },
        ),
        (
            OsStr::new("brewfile"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Brewfile",
            },
        ),
        (
            OsStr::new("bspwmrc"),
            DevIcon {
                icon: "",
                color: "#2f2f2f",

                name: "BSPWM",
            },
        ),
        (
            OsStr::new("build"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "BazelBuild",
            },
        ),
        (
            OsStr::new("checkhealth"),
            DevIcon {
                icon: "󰓙",
                color: "#75B4FB",

                name: "Checkhealth",
            },
        ),
        (
            OsStr::new("cmakelists.txt"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "CMakeLists",
            },
        ),
        (
            OsStr::new("commit_editmsg"),
            DevIcon {
                icon: "",
                color: "#f54d27",

                name: "GitCommit",
            },
        ),
        (
            OsStr::new("compose.yaml"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("compose.yml"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("config"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Config",
            },
        ),
        (
            OsStr::new("containerfile"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("copying"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "License",
            },
        ),
        (
            OsStr::new("copying.lesser"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "License",
            },
        ),
        (
            OsStr::new("docker-compose.yaml"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("docker-compose.yml"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("dockerfile"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("ext_typoscript_setup.txt"),
            DevIcon {
                icon: "",
                color: "#FF8700",

                name: "TypoScriptSetup",
            },
        ),
        (
            OsStr::new("favicon.ico"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Favicon",
            },
        ),
        (
            OsStr::new("fp-info-cache"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCadCache",
            },
        ),
        (
            OsStr::new("fp-lib-table"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCadFootprintTable",
            },
        ),
        (
            OsStr::new("FreeCAD.conf"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCADConfig",
            },
        ),
        (
            OsStr::new("gemfile$"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Gemfile",
            },
        ),
        (
            OsStr::new("gnumakefile"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Makefile",
            },
        ),
        (
            OsStr::new("gradlew"),
            DevIcon {
                icon: "",
                color: "#005f87",

                name: "GradleWrapperScript",
            },
        ),
        (
            OsStr::new("gradle.properties"),
            DevIcon {
                icon: "",
                color: "#005f87",

                name: "GradleProperties",
            },
        ),
        (
            OsStr::new("gradle-wrapper.properties"),
            DevIcon {
                icon: "",
                color: "#005f87",

                name: "GradleWrapperProperties",
            },
        ),
        (
            OsStr::new("groovy"),
            DevIcon {
                icon: "",
                color: "#4a687c",

                name: "Groovy",
            },
        ),
        (
            OsStr::new("gruntfile.babel.js"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Gruntfile",
            },
        ),
        (
            OsStr::new("gruntfile.coffee"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Gruntfile",
            },
        ),
        (
            OsStr::new("gruntfile.js"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Gruntfile",
            },
        ),
        (
            OsStr::new("gruntfile.ts"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Gruntfile",
            },
        ),
        (
            OsStr::new("gtkrc"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "GTK",
            },
        ),
        (
            OsStr::new("gulpfile.babel.js"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "Gulpfile",
            },
        ),
        (
            OsStr::new("gulpfile.coffee"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "Gulpfile",
            },
        ),
        (
            OsStr::new("gulpfile.js"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "Gulpfile",
            },
        ),
        (
            OsStr::new("gulpfile.ts"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "Gulpfile",
            },
        ),
        (
            OsStr::new("hyprland.conf"),
            DevIcon {
                icon: "",
                color: "#00aaae",

                name: "Hyprland",
            },
        ),
        (
            OsStr::new("i3blocks.conf"),
            DevIcon {
                icon: "",
                color: "#e8ebee",

                name: "i3",
            },
        ),
        (
            OsStr::new("i3status.conf"),
            DevIcon {
                icon: "",
                color: "#e8ebee",

                name: "i3",
            },
        ),
        (
            OsStr::new("cantorrc"),
            DevIcon {
                icon: "",
                color: "#1c99f3",

                name: "Cantorrc",
            },
        ),
        (
            OsStr::new("kalgebrarc"),
            DevIcon {
                icon: "",
                color: "#1c99f3",

                name: "Kalgebrarc",
            },
        ),
        (
            OsStr::new("kdeglobals"),
            DevIcon {
                icon: "",
                color: "#1c99f3",

                name: "KDEglobals",
            },
        ),
        (
            OsStr::new("kdenlive-layoutsrc"),
            DevIcon {
                icon: "",
                color: "#83b8f2",

                name: "KdenliveLayoutsrc",
            },
        ),
        (
            OsStr::new("kdenliverc"),
            DevIcon {
                icon: "",
                color: "#83b8f2",

                name: "Kdenliverc",
            },
        ),
        (
            OsStr::new("kritadisplayrc"),
            DevIcon {
                icon: "",
                color: "#f245fb",

                name: "Kritadisplayrc",
            },
        ),
        (
            OsStr::new("kritarc"),
            DevIcon {
                icon: "",
                color: "#f245fb",

                name: "Kritarc",
            },
        ),
        (
            OsStr::new("license"),
            DevIcon {
                icon: "",
                color: "#d0bf41",

                name: "License",
            },
        ),
        (
            OsStr::new("lxde-rc.xml"),
            DevIcon {
                icon: "",
                color: "#909090",

                name: "LXDEConfigFile",
            },
        ),
        (
            OsStr::new("lxqt.conf"),
            DevIcon {
                icon: "",
                color: "#0192d3",

                name: "LXQtConfigFile",
            },
        ),
        (
            OsStr::new("makefile"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Makefile",
            },
        ),
        (
            OsStr::new("mix.lock"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "MixLock",
            },
        ),
        (
            OsStr::new("mpv.conf"),
            DevIcon {
                icon: "",
                color: "#3b1342",

                name: "Mpv",
            },
        ),
        (
            OsStr::new("node_modules"),
            DevIcon {
                icon: "",
                color: "#E8274B",

                name: "NodeModules",
            },
        ),
        (
            OsStr::new("package.json"),
            DevIcon {
                icon: "",
                color: "#e8274b",

                name: "PackageJson",
            },
        ),
        (
            OsStr::new("package-lock.json"),
            DevIcon {
                icon: "",
                color: "#7a0d21",

                name: "PackageLockJson",
            },
        ),
        (
            OsStr::new("PKGBUILD"),
            DevIcon {
                icon: "",
                color: "#0f94d2",

                name: "PKGBUILD",
            },
        ),
        (
            OsStr::new("platformio.ini"),
            DevIcon {
                icon: "",
                color: "#f6822b",

                name: "Platformio",
            },
        ),
        (
            OsStr::new("pom.xml"),
            DevIcon {
                icon: "",
                color: "#7a0d21",

                name: "Maven",
            },
        ),
        (
            OsStr::new("procfile"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Procfile",
            },
        ),
        (
            OsStr::new("PrusaSlicer.ini"),
            DevIcon {
                icon: "",
                color: "#ec6b23",

                name: "PrusaSlicer",
            },
        ),
        (
            OsStr::new("PrusaSlicerGcodeViewer.ini"),
            DevIcon {
                icon: "",
                color: "#ec6b23",

                name: "PrusaSlicer",
            },
        ),
        (
            OsStr::new("py.typed"),
            DevIcon {
                icon: "",
                color: "#ffbc03",

                name: "Py.typed",
            },
        ),
        (
            OsStr::new("QtProject.conf"),
            DevIcon {
                icon: "",
                color: "#40cd52",

                name: "Qt",
            },
        ),
        (
            OsStr::new("r"),
            DevIcon {
                icon: "󰟔",
                color: "#2266ba",

                name: "R",
            },
        ),
        (
            OsStr::new("rakefile"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Rakefile",
            },
        ),
        (
            OsStr::new("rmd"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Rmd",
            },
        ),
        (
            OsStr::new("svelte.config.js"),
            DevIcon {
                icon: "",
                color: "#ff3e00",

                name: "SvelteConfig",
            },
        ),
        (
            OsStr::new("sxhkdrc"),
            DevIcon {
                icon: "",
                color: "#2f2f2f",

                name: "BSPWM",
            },
        ),
        (
            OsStr::new("sym-lib-table"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCadSymbolTable",
            },
        ),
        (
            OsStr::new("tailwind.config.js"),
            DevIcon {
                icon: "󱏿",
                color: "#20c2e3",

                name: "TailwindConfig",
            },
        ),
        (
            OsStr::new("tailwind.config.mjs"),
            DevIcon {
                icon: "󱏿",
                color: "#20c2e3",

                name: "TailwindConfig",
            },
        ),
        (
            OsStr::new("tailwind.config.ts"),
            DevIcon {
                icon: "󱏿",
                color: "#20c2e3",

                name: "TailwindConfig",
            },
        ),
        (
            OsStr::new("tmux.conf"),
            DevIcon {
                icon: "",
                color: "#14ba19",

                name: "Tmux",
            },
        ),
        (
            OsStr::new("tmux.conf.local"),
            DevIcon {
                icon: "",
                color: "#14ba19",

                name: "Tmux",
            },
        ),
        (
            OsStr::new("tsconfig.json"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "TSConfig",
            },
        ),
        (
            OsStr::new("unlicense"),
            DevIcon {
                icon: "",
                color: "#d0bf41",

                name: "License",
            },
        ),
        (
            OsStr::new("vagrantfile$"),
            DevIcon {
                icon: "",
                color: "#1563FF",

                name: "Vagrantfile",
            },
        ),
        (
            OsStr::new("vlcrc"),
            DevIcon {
                icon: "󰕼",
                color: "#ee7a00",

                name: "VLC",
            },
        ),
        (
            OsStr::new("webpack"),
            DevIcon {
                icon: "󰜫",
                color: "#519aba",

                name: "Webpack",
            },
        ),
        (
            OsStr::new("weston.ini"),
            DevIcon {
                icon: "",
                color: "#ffbb01",

                name: "Weston",
            },
        ),
        (
            OsStr::new("workspace"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "BazelWorkspace",
            },
        ),
        (
            OsStr::new("xmobarrc"),
            DevIcon {
                icon: "",
                color: "#fd4d5d",

                name: "xmonad",
            },
        ),
        (
            OsStr::new("xmobarrc.hs"),
            DevIcon {
                icon: "",
                color: "#fd4d5d",

                name: "xmonad",
            },
        ),
        (
            OsStr::new("xmonad.hs"),
            DevIcon {
                icon: "",
                color: "#fd4d5d",

                name: "xmonad",
            },
        ),
        (
            OsStr::new("xorg.conf"),
            DevIcon {
                icon: "",
                color: "#e54d18",

                name: "XorgConf",
            },
        ),
        (
            OsStr::new("xsettingsd.conf"),
            DevIcon {
                icon: "",
                color: "#e54d18",

                name: "XSettingsdConf",
            },
        ),
        (
            OsStr::new("build.zig.zon"),
            DevIcon {
                icon: "",
                color: "#f69a1b",

                name: "ZigObjectNotation",
            },
        ),
    ]);

    let dev_icon_from_extension: HashMap<&OsStr, DevIcon> = HashMap::from_iter([
        (
            OsStr::new("3gp"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "3gp",
            },
        ),
        (
            OsStr::new("3mf"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "3DObjectFile",
            },
        ),
        (
            OsStr::new("7z"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "7z",
            },
        ),
        (
            OsStr::new("a"),
            DevIcon {
                icon: "",
                color: "#dcddd6",

                name: "StaticLibraryArchive",
            },
        ),
        (
            OsStr::new("aac"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "AdvancedAudioCoding",
            },
        ),
        (
            OsStr::new("aif"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "AudioInterchangeFileFormat",
            },
        ),
        (
            OsStr::new("aiff"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "AudioInterchangeFileFormat",
            },
        ),
        (
            OsStr::new("ape"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "MonkeysAudio",
            },
        ),
        (
            OsStr::new("ai"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Ai",
            },
        ),
        (
            OsStr::new("android"),
            DevIcon {
                icon: "",
                color: "#34a853",

                name: "Android",
            },
        ),
        (
            OsStr::new("apk"),
            DevIcon {
                icon: "",
                color: "#34a853",

                name: "apk",
            },
        ),
        (
            OsStr::new("app"),
            DevIcon {
                icon: "",
                color: "#9F0500",

                name: "App",
            },
        ),
        (
            OsStr::new("applescript"),
            DevIcon {
                icon: "",
                color: "#6d8085",

                name: "AppleScript",
            },
        ),
        (
            OsStr::new("asc"),
            DevIcon {
                icon: "󰦝",
                color: "#576d7f",

                name: "Asc",
            },
        ),
        (
            OsStr::new("ass"),
            DevIcon {
                icon: "󰨖",
                color: "#ffb713",

                name: "Ass",
            },
        ),
        (
            OsStr::new("astro"),
            DevIcon {
                icon: "",
                color: "#e23f67",

                name: "Astro",
            },
        ),
        (
            OsStr::new("awk"),
            DevIcon {
                icon: "",
                color: "#4d5a5e",

                name: "Awk",
            },
        ),
        (
            OsStr::new("azcli"),
            DevIcon {
                icon: "",
                color: "#0078d4",

                name: "AzureCli",
            },
        ),
        (
            OsStr::new("bak"),
            DevIcon {
                icon: "󰁯",
                color: "#6d8086",

                name: "Backup",
            },
        ),
        (
            OsStr::new("bash"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Bash",
            },
        ),
        (
            OsStr::new("bat"),
            DevIcon {
                icon: "",
                color: "#C1F12E",

                name: "Bat",
            },
        ),
        (
            OsStr::new("bazel"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Bazel",
            },
        ),
        (
            OsStr::new("bib"),
            DevIcon {
                icon: "󱉟",
                color: "#cbcb41",

                name: "BibTeX",
            },
        ),
        (
            OsStr::new("bicep"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Bicep",
            },
        ),
        (
            OsStr::new("bicepparam"),
            DevIcon {
                icon: "",
                color: "#9f74b3",

                name: "BicepParameters",
            },
        ),
        (
            OsStr::new("bin"),
            DevIcon {
                icon: "",
                color: "#9F0500",

                name: "Bin",
            },
        ),
        (
            OsStr::new("blade.php"),
            DevIcon {
                icon: "",
                color: "#f05340",

                name: "Blade",
            },
        ),
        (
            OsStr::new("blend"),
            DevIcon {
                icon: "󰂫",
                color: "#ea7600",

                name: "Blender",
            },
        ),
        (
            OsStr::new("bmp"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Bmp",
            },
        ),
        (
            OsStr::new("blp"),
            DevIcon {
                icon: "󰺾",
                color: "#5796E2",

                name: "Blueprint",
            },
        ),
        (
            OsStr::new("brep"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "BoundaryRepresentation",
            },
        ),
        (
            OsStr::new("bz"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Bz",
            },
        ),
        (
            OsStr::new("bz2"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Bz2",
            },
        ),
        (
            OsStr::new("bz3"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Bz3",
            },
        ),
        (
            OsStr::new("bzl"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Bzl",
            },
        ),
        (
            OsStr::new("c"),
            DevIcon {
                icon: "",
                color: "#599eff",

                name: "C",
            },
        ),
        (
            OsStr::new("c++"),
            DevIcon {
                icon: "",
                color: "#f34b7d",

                name: "CPlusPlus",
            },
        ),
        (
            OsStr::new("cache"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "Cache",
            },
        ),
        (
            OsStr::new("cast"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "Asciinema",
            },
        ),
        (
            OsStr::new("cbl"),
            DevIcon {
                icon: "⚙",
                color: "#005ca5",

                name: "Cobol",
            },
        ),
        (
            OsStr::new("cc"),
            DevIcon {
                icon: "",
                color: "#f34b7d",

                name: "CPlusPlus",
            },
        ),
        (
            OsStr::new("ccm"),
            DevIcon {
                icon: "",
                color: "#f34b7d",

                name: "CPlusPlusModule",
            },
        ),
        (
            OsStr::new("cfg"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Configuration",
            },
        ),
        (
            OsStr::new("cjs"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Cjs",
            },
        ),
        (
            OsStr::new("clj"),
            DevIcon {
                icon: "",
                color: "#8dc149",

                name: "Clojure",
            },
        ),
        (
            OsStr::new("cljc"),
            DevIcon {
                icon: "",
                color: "#8dc149",

                name: "ClojureC",
            },
        ),
        (
            OsStr::new("cljs"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "ClojureJS",
            },
        ),
        (
            OsStr::new("cljd"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "ClojureDart",
            },
        ),
        (
            OsStr::new("cmake"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "CMake",
            },
        ),
        (
            OsStr::new("cob"),
            DevIcon {
                icon: "⚙",
                color: "#005ca5",

                name: "Cobol",
            },
        ),
        (
            OsStr::new("cobol"),
            DevIcon {
                icon: "⚙",
                color: "#005ca5",

                name: "Cobol",
            },
        ),
        (
            OsStr::new("coffee"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Coffee",
            },
        ),
        (
            OsStr::new("conf"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Conf",
            },
        ),
        (
            OsStr::new("config.ru"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "ConfigRu",
            },
        ),
        (
            OsStr::new("cp"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Cp",
            },
        ),
        (
            OsStr::new("cpp"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Cpp",
            },
        ),
        (
            OsStr::new("cppm"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Cppm",
            },
        ),
        (
            OsStr::new("cpy"),
            DevIcon {
                icon: "⚙",
                color: "#005ca5",

                name: "Cobol",
            },
        ),
        (
            OsStr::new("cr"),
            DevIcon {
                icon: "",
                color: "#c8c8c8",

                name: "Crystal",
            },
        ),
        (
            OsStr::new("crdownload"),
            DevIcon {
                icon: "",
                color: "#44cda8",

                name: "Crdownload",
            },
        ),
        (
            OsStr::new("cs"),
            DevIcon {
                icon: "󰌛",
                color: "#596706",

                name: "Cs",
            },
        ),
        (
            OsStr::new("csh"),
            DevIcon {
                icon: "",
                color: "#4d5a5e",

                name: "Csh",
            },
        ),
        (
            OsStr::new("cshtml"),
            DevIcon {
                icon: "󱦗",
                color: "#512bd4",

                name: "RazorPage",
            },
        ),
        (
            OsStr::new("cson"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Cson",
            },
        ),
        (
            OsStr::new("csproj"),
            DevIcon {
                icon: "󰪮",
                color: "#512bd4",

                name: "CSharpProject",
            },
        ),
        (
            OsStr::new("css"),
            DevIcon {
                icon: "",
                color: "#42a5f5",

                name: "Css",
            },
        ),
        (
            OsStr::new("csv"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Csv",
            },
        ),
        (
            OsStr::new("cts"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Cts",
            },
        ),
        (
            OsStr::new("cu"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "cuda",
            },
        ),
        (
            OsStr::new("cue"),
            DevIcon {
                icon: "󰲹",
                color: "#ed95ae",

                name: "Cue",
            },
        ),
        (
            OsStr::new("cuh"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "cudah",
            },
        ),
        (
            OsStr::new("cxx"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Cxx",
            },
        ),
        (
            OsStr::new("cxxm"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Cxxm",
            },
        ),
        (
            OsStr::new("d"),
            DevIcon {
                icon: "",
                color: "#427819",

                name: "D",
            },
        ),
        (
            OsStr::new("d.ts"),
            DevIcon {
                icon: "",
                color: "#d59855",

                name: "TypeScriptDeclaration",
            },
        ),
        (
            OsStr::new("dart"),
            DevIcon {
                icon: "",
                color: "#03589C",

                name: "Dart",
            },
        ),
        (
            OsStr::new("db"),
            DevIcon {
                icon: "",
                color: "#dad8d8",

                name: "Db",
            },
        ),
        (
            OsStr::new("dconf"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "Dconf",
            },
        ),
        (
            OsStr::new("desktop"),
            DevIcon {
                icon: "",
                color: "#563d7c",

                name: "DesktopEntry",
            },
        ),
        (
            OsStr::new("diff"),
            DevIcon {
                icon: "",
                color: "#41535b",

                name: "Diff",
            },
        ),
        (
            OsStr::new("dll"),
            DevIcon {
                icon: "",
                color: "#4d2c0b",

                name: "Dll",
            },
        ),
        (
            OsStr::new("doc"),
            DevIcon {
                icon: "󰈬",
                color: "#185abd",

                name: "Doc",
            },
        ),
        (
            OsStr::new("Dockerfile"),
            DevIcon {
                icon: "󰡨",
                color: "#458ee6",

                name: "Dockerfile",
            },
        ),
        (
            OsStr::new("docx"),
            DevIcon {
                icon: "󰈬",
                color: "#185abd",

                name: "Docx",
            },
        ),
        (
            OsStr::new("dot"),
            DevIcon {
                icon: "󱁉",
                color: "#30638e",

                name: "Dot",
            },
        ),
        (
            OsStr::new("download"),
            DevIcon {
                icon: "",
                color: "#44cda8",

                name: "Download",
            },
        ),
        (
            OsStr::new("drl"),
            DevIcon {
                icon: "",
                color: "#ffafaf",

                name: "Drools",
            },
        ),
        (
            OsStr::new("dropbox"),
            DevIcon {
                icon: "",
                color: "#0061FE",

                name: "Dropbox",
            },
        ),
        (
            OsStr::new("dump"),
            DevIcon {
                icon: "",
                color: "#dad8d8",

                name: "Dump",
            },
        ),
        (
            OsStr::new("dwg"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "AutoCADDwg",
            },
        ),
        (
            OsStr::new("dxf"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "AutoCADDxf",
            },
        ),
        (
            OsStr::new("ebook"),
            DevIcon {
                icon: "",
                color: "#eab16d",

                name: "Ebook",
            },
        ),
        (
            OsStr::new("edn"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Edn",
            },
        ),
        (
            OsStr::new("eex"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Eex",
            },
        ),
        (
            OsStr::new("ejs"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Ejs",
            },
        ),
        (
            OsStr::new("elf"),
            DevIcon {
                icon: "",
                color: "#9F0500",

                name: "Elf",
            },
        ),
        (
            OsStr::new("el"),
            DevIcon {
                icon: "",
                color: "#8172be",

                name: "Elisp",
            },
        ),
        (
            OsStr::new("elc"),
            DevIcon {
                icon: "",
                color: "#8172be",

                name: "Elisp",
            },
        ),
        (
            OsStr::new("elm"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Elm",
            },
        ),
        (
            OsStr::new("eln"),
            DevIcon {
                icon: "",
                color: "#8172be",

                name: "Elisp",
            },
        ),
        (
            OsStr::new("env"),
            DevIcon {
                icon: "",
                color: "#faf743",

                name: "Env",
            },
        ),
        (
            OsStr::new("eot"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "EmbeddedOpenTypeFont",
            },
        ),
        (
            OsStr::new("epp"),
            DevIcon {
                icon: "",
                color: "#FFA61A",

                name: "Epp",
            },
        ),
        (
            OsStr::new("epub"),
            DevIcon {
                icon: "",
                color: "#eab16d",

                name: "Epub",
            },
        ),
        (
            OsStr::new("erb"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Erb",
            },
        ),
        (
            OsStr::new("erl"),
            DevIcon {
                icon: "",
                color: "#B83998",

                name: "Erl",
            },
        ),
        (
            OsStr::new("ex"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Ex",
            },
        ),
        (
            OsStr::new("exe"),
            DevIcon {
                icon: "",
                color: "#9F0500",

                name: "Exe",
            },
        ),
        (
            OsStr::new("exs"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Exs",
            },
        ),
        (
            OsStr::new("f#"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Fsharp",
            },
        ),
        (
            OsStr::new("f3d"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Fusion360",
            },
        ),
        (
            OsStr::new("f90"),
            DevIcon {
                icon: "󱈚",
                color: "#734f96",

                name: "Fortran",
            },
        ),
        (
            OsStr::new("fbx"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "3DObjectFile",
            },
        ),
        (
            OsStr::new("fcbak"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fcmacro"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fcmat"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fcparam"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fcscript"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fcstd"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fcstd1"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fctb"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fctl"),
            DevIcon {
                icon: "",
                color: "#cb0d0d",

                name: "FreeCAD",
            },
        ),
        (
            OsStr::new("fdmdownload"),
            DevIcon {
                icon: "",
                color: "#44cda8",

                name: "Fdmdownload",
            },
        ),
        (
            OsStr::new("flac"),
            DevIcon {
                icon: "",
                color: "#0075aa",

                name: "FreeLosslessAudioCodec",
            },
        ),
        (
            OsStr::new("flc"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "FIGletFontControl",
            },
        ),
        (
            OsStr::new("flf"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "FIGletFontFormat",
            },
        ),
        (
            OsStr::new("fnl"),
            DevIcon {
                icon: "",
                color: "#fff3d7",

                name: "Fennel",
            },
        ),
        (
            OsStr::new("fish"),
            DevIcon {
                icon: "",
                color: "#4d5a5e",

                name: "Fish",
            },
        ),
        (
            OsStr::new("fs"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Fs",
            },
        ),
        (
            OsStr::new("fsi"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Fsi",
            },
        ),
        (
            OsStr::new("fsscript"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Fsscript",
            },
        ),
        (
            OsStr::new("fsx"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Fsx",
            },
        ),
        (
            OsStr::new("gcode"),
            DevIcon {
                icon: "󰐫",
                color: "#1471ad",

                name: "GCode",
            },
        ),
        (
            OsStr::new("gd"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "GDScript",
            },
        ),
        (
            OsStr::new("gemspec"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Gemspec",
            },
        ),
        (
            OsStr::new("gif"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Gif",
            },
        ),
        (
            OsStr::new("git"),
            DevIcon {
                icon: "",
                color: "#F14C28",

                name: "GitLogo",
            },
        ),
        (
            OsStr::new("glb"),
            DevIcon {
                icon: "",
                color: "#FFB13B",

                name: "BinaryGLTF",
            },
        ),
        (
            OsStr::new("gnumakefile"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Makefile",
            },
        ),
        (
            OsStr::new("go"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Go",
            },
        ),
        (
            OsStr::new("godot"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "GodotProject",
            },
        ),
        (
            OsStr::new("gql"),
            DevIcon {
                icon: "",
                color: "#e535ab",

                name: "GraphQL",
            },
        ),
        (
            OsStr::new("graphql"),
            DevIcon {
                icon: "",
                color: "#e535ab",

                name: "GraphQL",
            },
        ),
        (
            OsStr::new("gresource"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "GTK",
            },
        ),
        (
            OsStr::new("gv"),
            DevIcon {
                icon: "󱁉",
                color: "#30638e",

                name: "Gv",
            },
        ),
        (
            OsStr::new("gz"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Gz",
            },
        ),
        (
            OsStr::new("h"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "H",
            },
        ),
        (
            OsStr::new("haml"),
            DevIcon {
                icon: "",
                color: "#eaeae1",

                name: "Haml",
            },
        ),
        (
            OsStr::new("hx"),
            DevIcon {
                icon: "",
                color: "#ea8220",

                name: "Haxe",
            },
        ),
        (
            OsStr::new("hbs"),
            DevIcon {
                icon: "",
                color: "#f0772b",

                name: "Hbs",
            },
        ),
        (
            OsStr::new("hex"),
            DevIcon {
                icon: "",
                color: "#2e63ff",

                name: "Hexadecimal",
            },
        ),
        (
            OsStr::new("heex"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Heex",
            },
        ),
        (
            OsStr::new("hh"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Hh",
            },
        ),
        (
            OsStr::new("hpp"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Hpp",
            },
        ),
        (
            OsStr::new("hrl"),
            DevIcon {
                icon: "",
                color: "#B83998",

                name: "Hrl",
            },
        ),
        (
            OsStr::new("hs"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Hs",
            },
        ),
        (
            OsStr::new("htm"),
            DevIcon {
                icon: "",
                color: "#e34c26",

                name: "Htm",
            },
        ),
        (
            OsStr::new("html"),
            DevIcon {
                icon: "",
                color: "#e44d26",

                name: "Html",
            },
        ),
        (
            OsStr::new("huff"),
            DevIcon {
                icon: "󰡘",
                color: "#4242c7",

                name: "Huff",
            },
        ),
        (
            OsStr::new("hurl"),
            DevIcon {
                icon: "",
                color: "#ff0288",

                name: "Hurl",
            },
        ),
        (
            OsStr::new("hxx"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Hxx",
            },
        ),
        (
            OsStr::new("ixx"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Ixx",
            },
        ),
        (
            OsStr::new("ico"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Ico",
            },
        ),
        (
            OsStr::new("ical"),
            DevIcon {
                icon: "",
                color: "#2B2e83",

                name: "Ical",
            },
        ),
        (
            OsStr::new("icalendar"),
            DevIcon {
                icon: "",
                color: "#2B2e83",

                name: "Icalendar",
            },
        ),
        (
            OsStr::new("ics"),
            DevIcon {
                icon: "",
                color: "#2B2e83",

                name: "Ics",
            },
        ),
        (
            OsStr::new("ifb"),
            DevIcon {
                icon: "",
                color: "#2B2e83",

                name: "Ifb",
            },
        ),
        (
            OsStr::new("ifc"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Ifc",
            },
        ),
        (
            OsStr::new("ige"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Ige",
            },
        ),
        (
            OsStr::new("iges"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Iges",
            },
        ),
        (
            OsStr::new("igs"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Igs",
            },
        ),
        (
            OsStr::new("image"),
            DevIcon {
                icon: "",
                color: "#d0bec8",

                name: "Image",
            },
        ),
        (
            OsStr::new("img"),
            DevIcon {
                icon: "",
                color: "#d0bec8",

                name: "Img",
            },
        ),
        (
            OsStr::new("import"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "ImportConfiguration",
            },
        ),
        (
            OsStr::new("info"),
            DevIcon {
                icon: "",
                color: "#ffffcd",

                name: "Info",
            },
        ),
        (
            OsStr::new("ini"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Ini",
            },
        ),
        (
            OsStr::new("ino"),
            DevIcon {
                icon: "",
                color: "#56b6c2",

                name: "Arduino",
            },
        ),
        (
            OsStr::new("iso"),
            DevIcon {
                icon: "",
                color: "#d0bec8",

                name: "Iso",
            },
        ),
        (
            OsStr::new("ipynb"),
            DevIcon {
                icon: "",
                color: "#51a0cf",

                name: "Notebook",
            },
        ),
        (
            OsStr::new("java"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "Java",
            },
        ),
        (
            OsStr::new("jl"),
            DevIcon {
                icon: "",
                color: "#a270ba",

                name: "Jl",
            },
        ),
        (
            OsStr::new("jwmrc"),
            DevIcon {
                icon: "",
                color: "#0078cd",

                name: "JWM",
            },
        ),
        (
            OsStr::new("jpeg"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Jpeg",
            },
        ),
        (
            OsStr::new("jpg"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Jpg",
            },
        ),
        (
            OsStr::new("js"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Js",
            },
        ),
        (
            OsStr::new("json"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Json",
            },
        ),
        (
            OsStr::new("json5"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Json5",
            },
        ),
        (
            OsStr::new("jsonc"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "Jsonc",
            },
        ),
        (
            OsStr::new("jsx"),
            DevIcon {
                icon: "",
                color: "#20c2e3",

                name: "Jsx",
            },
        ),
        (
            OsStr::new("jxl"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "JpegXl",
            },
        ),
        (
            OsStr::new("kbx"),
            DevIcon {
                icon: "󰯄",
                color: "#737672",

                name: "Kbx",
            },
        ),
        (
            OsStr::new("kdb"),
            DevIcon {
                icon: "",
                color: "#529b34",

                name: "Kdb",
            },
        ),
        (
            OsStr::new("kdbx"),
            DevIcon {
                icon: "",
                color: "#529b34",

                name: "Kdbx",
            },
        ),
        (
            OsStr::new("kdenlive"),
            DevIcon {
                icon: "",
                color: "#83b8f2",

                name: "Kdenlive",
            },
        ),
        (
            OsStr::new("kdenlivetitle"),
            DevIcon {
                icon: "",
                color: "#83b8f2",

                name: "Kdenlive",
            },
        ),
        (
            OsStr::new("kicad_dru"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_mod"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_pcb"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_prl"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_pro"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_sch"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_sym"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("kicad_wks"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "KiCad",
            },
        ),
        (
            OsStr::new("ko"),
            DevIcon {
                icon: "",
                color: "#dcddd6",

                name: "LinuxKernelObject",
            },
        ),
        (
            OsStr::new("kpp"),
            DevIcon {
                icon: "",
                color: "#f245fb",

                name: "Krita",
            },
        ),
        (
            OsStr::new("kra"),
            DevIcon {
                icon: "",
                color: "#f245fb",

                name: "Krita",
            },
        ),
        (
            OsStr::new("krz"),
            DevIcon {
                icon: "",
                color: "#f245fb",

                name: "Krita",
            },
        ),
        (
            OsStr::new("ksh"),
            DevIcon {
                icon: "",
                color: "#4d5a5e",

                name: "Ksh",
            },
        ),
        (
            OsStr::new("kt"),
            DevIcon {
                icon: "",
                color: "#7F52FF",

                name: "Kotlin",
            },
        ),
        (
            OsStr::new("kts"),
            DevIcon {
                icon: "",
                color: "#7F52FF",

                name: "KotlinScript",
            },
        ),
        (
            OsStr::new("lck"),
            DevIcon {
                icon: "",
                color: "#bbbbbb",

                name: "Lock",
            },
        ),
        (
            OsStr::new("leex"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Leex",
            },
        ),
        (
            OsStr::new("less"),
            DevIcon {
                icon: "",
                color: "#563d7c",

                name: "Less",
            },
        ),
        (
            OsStr::new("lff"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "LibrecadFontFile",
            },
        ),
        (
            OsStr::new("lhs"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Lhs",
            },
        ),
        (
            OsStr::new("lib"),
            DevIcon {
                icon: "",
                color: "#4d2c0b",

                name: "Lib",
            },
        ),
        (
            OsStr::new("license"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "License",
            },
        ),
        (
            OsStr::new("liquid"),
            DevIcon {
                icon: "",
                color: "#95BF47",

                name: "Liquid",
            },
        ),
        (
            OsStr::new("lock"),
            DevIcon {
                icon: "",
                color: "#bbbbbb",

                name: "Lock",
            },
        ),
        (
            OsStr::new("log"),
            DevIcon {
                icon: "󰌱",
                color: "#dddddd",

                name: "Log",
            },
        ),
        (
            OsStr::new("lrc"),
            DevIcon {
                icon: "󰨖",
                color: "#ffb713",

                name: "Lrc",
            },
        ),
        (
            OsStr::new("lua"),
            DevIcon {
                icon: "",
                color: "#51a0cf",

                name: "Lua",
            },
        ),
        (
            OsStr::new("luac"),
            DevIcon {
                icon: "",
                color: "#51a0cf",

                name: "Lua",
            },
        ),
        (
            OsStr::new("luau"),
            DevIcon {
                icon: "",
                color: "#00a2ff",

                name: "Luau",
            },
        ),
        (
            OsStr::new("m3u"),
            DevIcon {
                icon: "󰲹",
                color: "#ed95ae",

                name: "M3u",
            },
        ),
        (
            OsStr::new("m3u8"),
            DevIcon {
                icon: "󰲹",
                color: "#ed95ae",

                name: "M3u8",
            },
        ),
        (
            OsStr::new("m4a"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "MPEG4",
            },
        ),
        (
            OsStr::new("m4v"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "M4V",
            },
        ),
        (
            OsStr::new("magnet"),
            DevIcon {
                icon: "",
                color: "#a51b16",

                name: "Magnet",
            },
        ),
        (
            OsStr::new("makefile"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Makefile",
            },
        ),
        (
            OsStr::new("markdown"),
            DevIcon {
                icon: "",
                color: "#dddddd",

                name: "Markdown",
            },
        ),
        (
            OsStr::new("material"),
            DevIcon {
                icon: "󰔉",
                color: "#B83998",

                name: "Material",
            },
        ),
        (
            OsStr::new("md"),
            DevIcon {
                icon: "",
                color: "#dddddd",

                name: "Md",
            },
        ),
        (
            OsStr::new("md5"),
            DevIcon {
                icon: "󰕥",
                color: "#8c86af",

                name: "Md5",
            },
        ),
        (
            OsStr::new("mdx"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Mdx",
            },
        ),
        (
            OsStr::new("mint"),
            DevIcon {
                icon: "󰌪",
                color: "#87c095",

                name: "Mint",
            },
        ),
        (
            OsStr::new("mjs"),
            DevIcon {
                icon: "",
                color: "#f1e05a",

                name: "Mjs",
            },
        ),
        (
            OsStr::new("mk"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Makefile",
            },
        ),
        (
            OsStr::new("mkv"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "Mkv",
            },
        ),
        (
            OsStr::new("ml"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Ml",
            },
        ),
        (
            OsStr::new("mli"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Mli",
            },
        ),
        (
            OsStr::new("m"),
            DevIcon {
                icon: "",
                color: "#599eff",

                name: "ObjectiveC",
            },
        ),
        (
            OsStr::new("mm"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "ObjectiveCPlusPlus",
            },
        ),
        (
            OsStr::new("mo"),
            DevIcon {
                icon: "∞",
                color: "#9772FB",

                name: "Motoko",
            },
        ),
        (
            OsStr::new("mobi"),
            DevIcon {
                icon: "",
                color: "#eab16d",

                name: "Mobi",
            },
        ),
        (
            OsStr::new("mov"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "MOV",
            },
        ),
        (
            OsStr::new("mp3"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "MPEGAudioLayerIII",
            },
        ),
        (
            OsStr::new("mp4"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "Mp4",
            },
        ),
        (
            OsStr::new("mpp"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Mpp",
            },
        ),
        (
            OsStr::new("msf"),
            DevIcon {
                icon: "",
                color: "#137be1",

                name: "Thunderbird",
            },
        ),
        (
            OsStr::new("mts"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Mts",
            },
        ),
        (
            OsStr::new("mustache"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Mustache",
            },
        ),
        (
            OsStr::new("nfo"),
            DevIcon {
                icon: "",
                color: "#ffffcd",

                name: "Nfo",
            },
        ),
        (
            OsStr::new("nim"),
            DevIcon {
                icon: "",
                color: "#f3d400",

                name: "Nim",
            },
        ),
        (
            OsStr::new("nix"),
            DevIcon {
                icon: "",
                color: "#7ebae4",

                name: "Nix",
            },
        ),
        (
            OsStr::new("nswag"),
            DevIcon {
                icon: "",
                color: "#85ea2d",

                name: "Nswag",
            },
        ),
        (
            OsStr::new("nu"),
            DevIcon {
                icon: ">",
                color: "#3aa675",

                name: "Nushell",
            },
        ),
        (
            OsStr::new("o"),
            DevIcon {
                icon: "",
                color: "#9F0500",

                name: "ObjectFile",
            },
        ),
        (
            OsStr::new("obj"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "3DObjectFile",
            },
        ),
        (
            OsStr::new("ogg"),
            DevIcon {
                icon: "",
                color: "#0075aa",

                name: "OggVorbis",
            },
        ),
        (
            OsStr::new("opus"),
            DevIcon {
                icon: "",
                color: "#0075aa",

                name: "OpusAudioFile",
            },
        ),
        (
            OsStr::new("org"),
            DevIcon {
                icon: "",
                color: "#77AA99",

                name: "OrgMode",
            },
        ),
        (
            OsStr::new("otf"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "OpenTypeFont",
            },
        ),
        (
            OsStr::new("out"),
            DevIcon {
                icon: "",
                color: "#9F0500",

                name: "Out",
            },
        ),
        (
            OsStr::new("part"),
            DevIcon {
                icon: "",
                color: "#44cda8",

                name: "Part",
            },
        ),
        (
            OsStr::new("patch"),
            DevIcon {
                icon: "",
                color: "#41535b",

                name: "Patch",
            },
        ),
        (
            OsStr::new("pck"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "PackedResource",
            },
        ),
        (
            OsStr::new("pcm"),
            DevIcon {
                icon: "",
                color: "#0075aa",

                name: "PulseCodeModulation",
            },
        ),
        (
            OsStr::new("pdf"),
            DevIcon {
                icon: "",
                color: "#b30b00",

                name: "Pdf",
            },
        ),
        (
            OsStr::new("php"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Php",
            },
        ),
        (
            OsStr::new("pl"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Pl",
            },
        ),
        (
            OsStr::new("pls"),
            DevIcon {
                icon: "󰲹",
                color: "#ed95ae",

                name: "Pls",
            },
        ),
        (
            OsStr::new("ply"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "3DObjectFile",
            },
        ),
        (
            OsStr::new("pm"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Pm",
            },
        ),
        (
            OsStr::new("png"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Png",
            },
        ),
        (
            OsStr::new("po"),
            DevIcon {
                icon: "",
                color: "#2596be",

                name: "Localization",
            },
        ),
        (
            OsStr::new("pot"),
            DevIcon {
                icon: "",
                color: "#2596be",

                name: "Localization",
            },
        ),
        (
            OsStr::new("pp"),
            DevIcon {
                icon: "",
                color: "#FFA61A",

                name: "Pp",
            },
        ),
        (
            OsStr::new("ppt"),
            DevIcon {
                icon: "󰈧",
                color: "#cb4a32",

                name: "Ppt",
            },
        ),
        (
            OsStr::new("prisma"),
            DevIcon {
                icon: "",
                color: "#5a67d8",

                name: "Prisma",
            },
        ),
        (
            OsStr::new("pro"),
            DevIcon {
                icon: "",
                color: "#e4b854",

                name: "Prolog",
            },
        ),
        (
            OsStr::new("ps1"),
            DevIcon {
                icon: "󰨊",
                color: "#4273ca",

                name: "PsScriptfile",
            },
        ),
        (
            OsStr::new("psd1"),
            DevIcon {
                icon: "󰨊",
                color: "#6975c4",

                name: "PsManifestfile",
            },
        ),
        (
            OsStr::new("psm1"),
            DevIcon {
                icon: "󰨊",
                color: "#6975c4",

                name: "PsScriptModulefile",
            },
        ),
        (
            OsStr::new("psb"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Psb",
            },
        ),
        (
            OsStr::new("psd"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Psd",
            },
        ),
        (
            OsStr::new("pub"),
            DevIcon {
                icon: "󰷖",
                color: "#e3c58e",

                name: "Pub",
            },
        ),
        (
            OsStr::new("pxd"),
            DevIcon {
                icon: "",
                color: "#5aa7e4",

                name: "Pxd",
            },
        ),
        (
            OsStr::new("pxi"),
            DevIcon {
                icon: "",
                color: "#5aa7e4",

                name: "Pxi",
            },
        ),
        (
            OsStr::new("py"),
            DevIcon {
                icon: "",
                color: "#ffbc03",

                name: "Py",
            },
        ),
        (
            OsStr::new("pyc"),
            DevIcon {
                icon: "",
                color: "#ffe291",

                name: "Pyc",
            },
        ),
        (
            OsStr::new("pyd"),
            DevIcon {
                icon: "",
                color: "#ffe291",

                name: "Pyd",
            },
        ),
        (
            OsStr::new("pyi"),
            DevIcon {
                icon: "",
                color: "#ffbc03",

                name: "Pyi",
            },
        ),
        (
            OsStr::new("pyo"),
            DevIcon {
                icon: "",
                color: "#ffe291",

                name: "Pyo",
            },
        ),
        (
            OsStr::new("pyx"),
            DevIcon {
                icon: "",
                color: "#5aa7e4",

                name: "Pyx",
            },
        ),
        (
            OsStr::new("qm"),
            DevIcon {
                icon: "",
                color: "#2596be",

                name: "Localization",
            },
        ),
        (
            OsStr::new("qml"),
            DevIcon {
                icon: "",
                color: "#40cd52",

                name: "Qt",
            },
        ),
        (
            OsStr::new("qrc"),
            DevIcon {
                icon: "",
                color: "#40cd52",

                name: "Qt",
            },
        ),
        (
            OsStr::new("qss"),
            DevIcon {
                icon: "",
                color: "#40cd52",

                name: "Qt",
            },
        ),
        (
            OsStr::new("query"),
            DevIcon {
                icon: "",
                color: "#90a850",

                name: "Query",
            },
        ),
        (
            OsStr::new("r"),
            DevIcon {
                icon: "󰟔",
                color: "#2266ba",

                name: "R",
            },
        ),
        (
            OsStr::new("rake"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Rake",
            },
        ),
        (
            OsStr::new("rar"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Rar",
            },
        ),
        (
            OsStr::new("razor"),
            DevIcon {
                icon: "󱦘",
                color: "#512bd4",

                name: "RazorPage",
            },
        ),
        (
            OsStr::new("rb"),
            DevIcon {
                icon: "",
                color: "#701516",

                name: "Rb",
            },
        ),
        (
            OsStr::new("res"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "ReScript",
            },
        ),
        (
            OsStr::new("resi"),
            DevIcon {
                icon: "",
                color: "#f55385",

                name: "ReScriptInterface",
            },
        ),
        (
            OsStr::new("rlib"),
            DevIcon {
                icon: "",
                color: "#dea584",

                name: "Rlib",
            },
        ),
        (
            OsStr::new("rmd"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Rmd",
            },
        ),
        (
            OsStr::new("rproj"),
            DevIcon {
                icon: "󰗆",
                color: "#358a5b",

                name: "Rproj",
            },
        ),
        (
            OsStr::new("rs"),
            DevIcon {
                icon: "",
                color: "#dea584",

                name: "Rs",
            },
        ),
        (
            OsStr::new("rss"),
            DevIcon {
                icon: "",
                color: "#FB9D3B",

                name: "Rss",
            },
        ),
        (
            OsStr::new("sass"),
            DevIcon {
                icon: "",
                color: "#f55385",

                name: "Sass",
            },
        ),
        (
            OsStr::new("sbt"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "sbt",
            },
        ),
        (
            OsStr::new("scad"),
            DevIcon {
                icon: "",
                color: "#f9d72c",

                name: "OpenSCAD",
            },
        ),
        (
            OsStr::new("scala"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "Scala",
            },
        ),
        (
            OsStr::new("sc"),
            DevIcon {
                icon: "",
                color: "#cc3e44",

                name: "ScalaScript",
            },
        ),
        (
            OsStr::new("scm"),
            DevIcon {
                icon: "󰘧",
                color: "#eeeeee",

                name: "Scheme",
            },
        ),
        (
            OsStr::new("scss"),
            DevIcon {
                icon: "",
                color: "#f55385",

                name: "Scss",
            },
        ),
        (
            OsStr::new("sh"),
            DevIcon {
                icon: "",
                color: "#4d5a5e",

                name: "Sh",
            },
        ),
        (
            OsStr::new("sha1"),
            DevIcon {
                icon: "󰕥",
                color: "#8c86af",

                name: "Sha1",
            },
        ),
        (
            OsStr::new("sha224"),
            DevIcon {
                icon: "󰕥",
                color: "#8c86af",

                name: "Sha224",
            },
        ),
        (
            OsStr::new("sha256"),
            DevIcon {
                icon: "󰕥",
                color: "#8c86af",

                name: "Sha256",
            },
        ),
        (
            OsStr::new("sha384"),
            DevIcon {
                icon: "󰕥",
                color: "#8c86af",

                name: "Sha384",
            },
        ),
        (
            OsStr::new("sha512"),
            DevIcon {
                icon: "󰕥",
                color: "#8c86af",

                name: "Sha512",
            },
        ),
        (
            OsStr::new("sig"),
            DevIcon {
                icon: "λ",
                color: "#e37933",

                name: "Sig",
            },
        ),
        (
            OsStr::new("signature"),
            DevIcon {
                icon: "λ",
                color: "#e37933",

                name: "Signature",
            },
        ),
        (
            OsStr::new("skp"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "SketchUp",
            },
        ),
        (
            OsStr::new("sldasm"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "SolidWorksAsm",
            },
        ),
        (
            OsStr::new("sldprt"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "SolidWorksPrt",
            },
        ),
        (
            OsStr::new("slim"),
            DevIcon {
                icon: "",
                color: "#e34c26",

                name: "Slim",
            },
        ),
        (
            OsStr::new("sln"),
            DevIcon {
                icon: "",
                color: "#854CC7",

                name: "Sln",
            },
        ),
        (
            OsStr::new("slvs"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "SolveSpace",
            },
        ),
        (
            OsStr::new("sml"),
            DevIcon {
                icon: "λ",
                color: "#e37933",

                name: "Sml",
            },
        ),
        (
            OsStr::new("so"),
            DevIcon {
                icon: "",
                color: "#dcddd6",

                name: "SharedObject",
            },
        ),
        (
            OsStr::new("sol"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Solidity",
            },
        ),
        (
            OsStr::new("spec.js"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "SpecJs",
            },
        ),
        (
            OsStr::new("spec.jsx"),
            DevIcon {
                icon: "",
                color: "#20c2e3",

                name: "JavaScriptReactSpec",
            },
        ),
        (
            OsStr::new("spec.ts"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "SpecTs",
            },
        ),
        (
            OsStr::new("spec.tsx"),
            DevIcon {
                icon: "",
                color: "#1354bf",

                name: "TypeScriptReactSpec",
            },
        ),
        (
            OsStr::new("sql"),
            DevIcon {
                icon: "",
                color: "#dad8d8",

                name: "Sql",
            },
        ),
        (
            OsStr::new("sqlite"),
            DevIcon {
                icon: "",
                color: "#dad8d8",

                name: "Sql",
            },
        ),
        (
            OsStr::new("sqlite3"),
            DevIcon {
                icon: "",
                color: "#dad8d8",

                name: "Sql",
            },
        ),
        (
            OsStr::new("srt"),
            DevIcon {
                icon: "󰨖",
                color: "#ffb713",

                name: "Srt",
            },
        ),
        (
            OsStr::new("ssa"),
            DevIcon {
                icon: "󰨖",
                color: "#ffb713",

                name: "Ssa",
            },
        ),
        (
            OsStr::new("stl"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "3DObjectFile",
            },
        ),
        (
            OsStr::new("strings"),
            DevIcon {
                icon: "",
                color: "#2596be",

                name: "Localization",
            },
        ),
        (
            OsStr::new("ste"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Ste",
            },
        ),
        (
            OsStr::new("step"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Step",
            },
        ),
        (
            OsStr::new("stp"),
            DevIcon {
                icon: "󰻫",
                color: "#839463",

                name: "Stp",
            },
        ),
        (
            OsStr::new("styl"),
            DevIcon {
                icon: "",
                color: "#8dc149",

                name: "Styl",
            },
        ),
        (
            OsStr::new("sub"),
            DevIcon {
                icon: "󰨖",
                color: "#ffb713",

                name: "Sub",
            },
        ),
        (
            OsStr::new("sublime"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Sublime",
            },
        ),
        (
            OsStr::new("suo"),
            DevIcon {
                icon: "",
                color: "#854CC7",

                name: "Suo",
            },
        ),
        (
            OsStr::new("sv"),
            DevIcon {
                icon: "󰍛",
                color: "#019833",

                name: "SystemVerilog",
            },
        ),
        (
            OsStr::new("svelte"),
            DevIcon {
                icon: "",
                color: "#ff3e00",

                name: "Svelte",
            },
        ),
        (
            OsStr::new("svh"),
            DevIcon {
                icon: "󰍛",
                color: "#019833",

                name: "SystemVerilog",
            },
        ),
        (
            OsStr::new("svg"),
            DevIcon {
                icon: "󰜡",
                color: "#FFB13B",

                name: "Svg",
            },
        ),
        (
            OsStr::new("swift"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Swift",
            },
        ),
        (
            OsStr::new("t"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Tor",
            },
        ),
        (
            OsStr::new("tbc"),
            DevIcon {
                icon: "󰛓",
                color: "#1e5cb3",

                name: "Tcl",
            },
        ),
        (
            OsStr::new("tcl"),
            DevIcon {
                icon: "󰛓",
                color: "#1e5cb3",

                name: "Tcl",
            },
        ),
        (
            OsStr::new("templ"),
            DevIcon {
                icon: "",
                color: "#dbbd30",

                name: "Templ",
            },
        ),
        (
            OsStr::new("terminal"),
            DevIcon {
                icon: "",
                color: "#31B53E",

                name: "Terminal",
            },
        ),
        (
            OsStr::new("test.js"),
            DevIcon {
                icon: "",
                color: "#cbcb41",

                name: "TestJs",
            },
        ),
        (
            OsStr::new("test.jsx"),
            DevIcon {
                icon: "",
                color: "#20c2e3",

                name: "JavaScriptReactTest",
            },
        ),
        (
            OsStr::new("test.ts"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "TestTs",
            },
        ),
        (
            OsStr::new("test.tsx"),
            DevIcon {
                icon: "",
                color: "#1354bf",

                name: "TypeScriptReactTest",
            },
        ),
        (
            OsStr::new("tex"),
            DevIcon {
                icon: "",
                color: "#3D6117",

                name: "Tex",
            },
        ),
        (
            OsStr::new("tf"),
            DevIcon {
                icon: "",
                color: "#5F43E9",

                name: "Terraform",
            },
        ),
        (
            OsStr::new("tfvars"),
            DevIcon {
                icon: "",
                color: "#5F43E9",

                name: "TFVars",
            },
        ),
        (
            OsStr::new("tgz"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Tgz",
            },
        ),
        (
            OsStr::new("tmux"),
            DevIcon {
                icon: "",
                color: "#14ba19",

                name: "Tmux",
            },
        ),
        (
            OsStr::new("toml"),
            DevIcon {
                icon: "",
                color: "#9c4221",

                name: "Toml",
            },
        ),
        (
            OsStr::new("torrent"),
            DevIcon {
                icon: "",
                color: "#44cda8",

                name: "Torrent",
            },
        ),
        (
            OsStr::new("tres"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "GodotTextResource",
            },
        ),
        (
            OsStr::new("ts"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "TypeScript",
            },
        ),
        (
            OsStr::new("tscn"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "GodotTextScene",
            },
        ),
        (
            OsStr::new("tsconfig"),
            DevIcon {
                icon: "",
                color: "#FF8700",

                name: "TypoScriptConfig",
            },
        ),
        (
            OsStr::new("tsx"),
            DevIcon {
                icon: "",
                color: "#1354bf",

                name: "Tsx",
            },
        ),
        (
            OsStr::new("ttf"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "TrueTypeFont",
            },
        ),
        (
            OsStr::new("twig"),
            DevIcon {
                icon: "",
                color: "#8dc149",

                name: "Twig",
            },
        ),
        (
            OsStr::new("txz"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Txz",
            },
        ),
        (
            OsStr::new("typoscript"),
            DevIcon {
                icon: "",
                color: "#FF8700",

                name: "TypoScript",
            },
        ),
        (
            OsStr::new("txt"),
            DevIcon {
                icon: "󰈙",
                color: "#89e051",

                name: "Txt",
            },
        ),
        (
            OsStr::new("ui"),
            DevIcon {
                icon: "",
                color: "#0c306e",

                name: "UI",
            },
        ),
        (
            OsStr::new("v"),
            DevIcon {
                icon: "󰍛",
                color: "#019833",

                name: "Verilog",
            },
        ),
        (
            OsStr::new("vala"),
            DevIcon {
                icon: "",
                color: "#7239b3",

                name: "Vala",
            },
        ),
        (
            OsStr::new("vh"),
            DevIcon {
                icon: "󰍛",
                color: "#019833",

                name: "Verilog",
            },
        ),
        (
            OsStr::new("vhd"),
            DevIcon {
                icon: "󰍛",
                color: "#019833",

                name: "VHDL",
            },
        ),
        (
            OsStr::new("vhdl"),
            DevIcon {
                icon: "󰍛",
                color: "#019833",

                name: "VHDL",
            },
        ),
        (
            OsStr::new("vim"),
            DevIcon {
                icon: "",
                color: "#019833",

                name: "Vim",
            },
        ),
        (
            OsStr::new("vsh"),
            DevIcon {
                icon: "",
                color: "#5d87bf",

                name: "Vlang",
            },
        ),
        (
            OsStr::new("vsix"),
            DevIcon {
                icon: "",
                color: "#854CC7",

                name: "Vsix",
            },
        ),
        (
            OsStr::new("vue"),
            DevIcon {
                icon: "",
                color: "#8dc149",

                name: "Vue",
            },
        ),
        (
            OsStr::new("wasm"),
            DevIcon {
                icon: "",
                color: "#5c4cdb",

                name: "Wasm",
            },
        ),
        (
            OsStr::new("wav"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "WaveformAudioFile",
            },
        ),
        (
            OsStr::new("webm"),
            DevIcon {
                icon: "",
                color: "#FD971F",

                name: "Webm",
            },
        ),
        (
            OsStr::new("webmanifest"),
            DevIcon {
                icon: "",
                color: "#f1e05a",

                name: "Webmanifest",
            },
        ),
        (
            OsStr::new("webp"),
            DevIcon {
                icon: "",
                color: "#a074c4",

                name: "Webp",
            },
        ),
        (
            OsStr::new("webpack"),
            DevIcon {
                icon: "󰜫",
                color: "#519aba",

                name: "Webpack",
            },
        ),
        (
            OsStr::new("wma"),
            DevIcon {
                icon: "",
                color: "#00afff",

                name: "WindowsMediaAudio",
            },
        ),
        (
            OsStr::new("woff"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "WebOpenFontFormat",
            },
        ),
        (
            OsStr::new("woff2"),
            DevIcon {
                icon: "",
                color: "#ECECEC",

                name: "WebOpenFontFormat",
            },
        ),
        (
            OsStr::new("wrl"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "VRML",
            },
        ),
        (
            OsStr::new("wrz"),
            DevIcon {
                icon: "󰆧",
                color: "#888888",

                name: "VRML",
            },
        ),
        (
            OsStr::new("x"),
            DevIcon {
                icon: "",
                color: "#599eff",

                name: "Logos",
            },
        ),
        (
            OsStr::new("xm"),
            DevIcon {
                icon: "",
                color: "#519aba",

                name: "Logos",
            },
        ),
        (
            OsStr::new("xaml"),
            DevIcon {
                icon: "󰙳",
                color: "#512bd4",

                name: "Xaml",
            },
        ),
        (
            OsStr::new("xcf"),
            DevIcon {
                icon: "",
                color: "#635b46",

                name: "GIMP",
            },
        ),
        (
            OsStr::new("xcplayground"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "XcPlayground",
            },
        ),
        (
            OsStr::new("xcstrings"),
            DevIcon {
                icon: "",
                color: "#2596be",

                name: "XcLocalization",
            },
        ),
        (
            OsStr::new("xls"),
            DevIcon {
                icon: "󰈛",
                color: "#207245",

                name: "Xls",
            },
        ),
        (
            OsStr::new("xlsx"),
            DevIcon {
                icon: "󰈛",
                color: "#207245",

                name: "Xlsx",
            },
        ),
        (
            OsStr::new("xml"),
            DevIcon {
                icon: "󰗀",
                color: "#e37933",

                name: "Xml",
            },
        ),
        (
            OsStr::new("xpi"),
            DevIcon {
                icon: "",
                color: "#ff1b01",

                name: "Xpi",
            },
        ),
        (
            OsStr::new("xul"),
            DevIcon {
                icon: "",
                color: "#e37933",

                name: "Xul",
            },
        ),
        (
            OsStr::new("xz"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Xz",
            },
        ),
        (
            OsStr::new("yaml"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Yaml",
            },
        ),
        (
            OsStr::new("yml"),
            DevIcon {
                icon: "",
                color: "#6d8086",

                name: "Yml",
            },
        ),
        (
            OsStr::new("zig"),
            DevIcon {
                icon: "",
                color: "#f69a1b",

                name: "Zig",
            },
        ),
        (
            OsStr::new("zip"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Zip",
            },
        ),
        (
            OsStr::new("zsh"),
            DevIcon {
                icon: "",
                color: "#89e051",

                name: "Zsh",
            },
        ),
        (
            OsStr::new("zst"),
            DevIcon {
                icon: "",
                color: "#eca517",

                name: "Zst",
            },
        ),
    ]);

    let dev_icon_from_os = HashMap::from_iter([
        (
            OsStr::new("apple"),
            DevIcon {
                icon: "",
                color: "#A2AAAD",

                name: "Apple",
            },
        ),
        (
            OsStr::new("windows"),
            DevIcon {
                icon: "",
                color: "#00A4EF",

                name: "Windows",
            },
        ),
        (
            OsStr::new("linux"),
            DevIcon {
                icon: "",
                color: "#fdfdfb",

                name: "Linux",
            },
        ),
        (
            OsStr::new("alma"),
            DevIcon {
                icon: "",
                color: "#ff4649",

                name: "Almalinux",
            },
        ),
        (
            OsStr::new("alpine"),
            DevIcon {
                icon: "",
                color: "#0d597f",

                name: "Alpine",
            },
        ),
        (
            OsStr::new("aosc"),
            DevIcon {
                icon: "",
                color: "#c00000",

                name: "AOSC",
            },
        ),
        (
            OsStr::new("arch"),
            DevIcon {
                icon: "󰣇",
                color: "#0f94d2",

                name: "Arch",
            },
        ),
        (
            OsStr::new("archcraft"),
            DevIcon {
                icon: "",
                color: "#86bba3",

                name: "Archcraft",
            },
        ),
        (
            OsStr::new("archlabs"),
            DevIcon {
                icon: "",
                color: "#503f42",

                name: "Archlabs",
            },
        ),
        (
            OsStr::new("arcolinux"),
            DevIcon {
                icon: "",
                color: "#6690eb",

                name: "ArcoLinux",
            },
        ),
        (
            OsStr::new("artix"),
            DevIcon {
                icon: "",
                color: "#41b4d7",

                name: "Artix",
            },
        ),
        (
            OsStr::new("biglinux"),
            DevIcon {
                icon: "",
                color: "#189fc8",

                name: "BigLinux",
            },
        ),
        (
            OsStr::new("centos"),
            DevIcon {
                icon: "",
                color: "#a2518d",

                name: "Centos",
            },
        ),
        (
            OsStr::new("crystallinux"),
            DevIcon {
                icon: "",
                color: "#a900ff",

                name: "CrystalLinux",
            },
        ),
        (
            OsStr::new("debian"),
            DevIcon {
                icon: "",
                color: "#a80030",

                name: "Debian",
            },
        ),
        (
            OsStr::new("deepin"),
            DevIcon {
                icon: "",
                color: "#2ca7f8",

                name: "Deepin",
            },
        ),
        (
            OsStr::new("devuan"),
            DevIcon {
                icon: "",
                color: "#404a52",

                name: "Devuan",
            },
        ),
        (
            OsStr::new("elementary"),
            DevIcon {
                icon: "",
                color: "#5890c2",

                name: "Elementary",
            },
        ),
        (
            OsStr::new("endeavour"),
            DevIcon {
                icon: "",
                color: "#7b3db9",

                name: "Endeavour",
            },
        ),
        (
            OsStr::new("fedora"),
            DevIcon {
                icon: "",
                color: "#072a5e",

                name: "Fedora",
            },
        ),
        (
            OsStr::new("freebsd"),
            DevIcon {
                icon: "",
                color: "#c90f02",

                name: "FreeBSD",
            },
        ),
        (
            OsStr::new("garuda"),
            DevIcon {
                icon: "",
                color: "#2974e1",

                name: "GarudaLinux",
            },
        ),
        (
            OsStr::new("gentoo"),
            DevIcon {
                icon: "󰣨",
                color: "#b1abce",

                name: "Gentoo",
            },
        ),
        (
            OsStr::new("guix"),
            DevIcon {
                icon: "",
                color: "#ffcc00",

                name: "Guix",
            },
        ),
        (
            OsStr::new("hyperbola"),
            DevIcon {
                icon: "",
                color: "#c0c0c0",

                name: "HyperbolaGNULinuxLibre",
            },
        ),
        (
            OsStr::new("illumos"),
            DevIcon {
                icon: "",
                color: "#ff430f",

                name: "Illumos",
            },
        ),
        (
            OsStr::new("kali"),
            DevIcon {
                icon: "",
                color: "#2777ff",

                name: "Kali",
            },
        ),
        (
            OsStr::new("kdeneon"),
            DevIcon {
                icon: "",
                color: "#20a6a4",

                name: "KDEneon",
            },
        ),
        (
            OsStr::new("kubuntu"),
            DevIcon {
                icon: "",
                color: "#007ac2",

                name: "Kubuntu",
            },
        ),
        (
            OsStr::new("locos"),
            DevIcon {
                icon: "",
                color: "#fab402",

                name: "LocOS",
            },
        ),
        (
            OsStr::new("lxle"),
            DevIcon {
                icon: "",
                color: "#474747",

                name: "LXLE",
            },
        ),
        (
            OsStr::new("mint"),
            DevIcon {
                icon: "󰣭",
                color: "#66af3d",

                name: "Mint",
            },
        ),
        (
            OsStr::new("mageia"),
            DevIcon {
                icon: "",
                color: "#2397d4",

                name: "Mageia",
            },
        ),
        (
            OsStr::new("manjaro"),
            DevIcon {
                icon: "",
                color: "#33b959",

                name: "Manjaro",
            },
        ),
        (
            OsStr::new("mxlinux"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "MXLinux",
            },
        ),
        (
            OsStr::new("nixos"),
            DevIcon {
                icon: "",
                color: "#7ab1db",

                name: "NixOS",
            },
        ),
        (
            OsStr::new("openbsd"),
            DevIcon {
                icon: "",
                color: "#f2ca30",

                name: "OpenBSD",
            },
        ),
        (
            OsStr::new("opensuse"),
            DevIcon {
                icon: "",
                color: "#6fb424",

                name: "openSUSE",
            },
        ),
        (
            OsStr::new("parabola"),
            DevIcon {
                icon: "",
                color: "#797dac",

                name: "ParabolaGNULinuxLibre",
            },
        ),
        (
            OsStr::new("parrot"),
            DevIcon {
                icon: "",
                color: "#54deff",

                name: "Parrot",
            },
        ),
        (
            OsStr::new("pop_os"),
            DevIcon {
                icon: "",
                color: "#48b9c7",

                name: "Pop_OS",
            },
        ),
        (
            OsStr::new("postmarketos"),
            DevIcon {
                icon: "",
                color: "#009900",

                name: "postmarketOS",
            },
        ),
        (
            OsStr::new("puppylinux"),
            DevIcon {
                icon: "",
                color: "#a2aeb9",

                name: "PuppyLinux",
            },
        ),
        (
            OsStr::new("qubesos"),
            DevIcon {
                icon: "",
                color: "#3774d8",

                name: "QubesOS",
            },
        ),
        (
            OsStr::new("raspberry_pi"),
            DevIcon {
                icon: "",
                color: "#be1848",

                name: "RaspberryPiOS",
            },
        ),
        (
            OsStr::new("redhat"),
            DevIcon {
                icon: "󱄛",
                color: "#EE0000",

                name: "Redhat",
            },
        ),
        (
            OsStr::new("rocky"),
            DevIcon {
                icon: "",
                color: "#0fb37d",

                name: "RockyLinux",
            },
        ),
        (
            OsStr::new("sabayon"),
            DevIcon {
                icon: "",
                color: "#c6c6c6",

                name: "Sabayon",
            },
        ),
        (
            OsStr::new("slackware"),
            DevIcon {
                icon: "",
                color: "#475fa9",

                name: "Slackware",
            },
        ),
        (
            OsStr::new("solus"),
            DevIcon {
                icon: "",
                color: "#4b5163",

                name: "Solus",
            },
        ),
        (
            OsStr::new("tails"),
            DevIcon {
                icon: "",
                color: "#56347c",

                name: "Tails",
            },
        ),
        (
            OsStr::new("trisquel"),
            DevIcon {
                icon: "",
                color: "#0f58b6",

                name: "TrisquelGNULinux",
            },
        ),
        (
            OsStr::new("ubuntu"),
            DevIcon {
                icon: "",
                color: "#dd4814",

                name: "Ubuntu",
            },
        ),
        (
            OsStr::new("vanillaos"),
            DevIcon {
                icon: "",
                color: "#fabd4d",

                name: "VanillaOS",
            },
        ),
        (
            OsStr::new("void"),
            DevIcon {
                icon: "",
                color: "#295340",

                name: "Void",
            },
        ),
        (
            OsStr::new("xerolinux"),
            DevIcon {
                icon: "",
                color: "#888fe2",

                name: "XeroLinux",
            },
        ),
        (
            OsStr::new("zorin"),
            DevIcon {
                icon: "",
                color: "#14a1e8",

                name: "Zorin",
            },
        ),
    ]);

    let dev_icon_from_de = HashMap::from_iter([
        (
            OsStr::new("budgie"),
            DevIcon {
                icon: "",
                color: "#4e5361",

                name: "Budgie",
            },
        ),
        (
            OsStr::new("cinnamon"),
            DevIcon {
                icon: "",
                color: "#dc682e",

                name: "Cinnamon",
            },
        ),
        (
            OsStr::new("gnome"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "GNOME",
            },
        ),
        (
            OsStr::new("lxde"),
            DevIcon {
                icon: "",
                color: "#a4a4a4",

                name: "LXDE",
            },
        ),
        (
            OsStr::new("lxqt"),
            DevIcon {
                icon: "",
                color: "#0191d2",

                name: "LXQt",
            },
        ),
        (
            OsStr::new("mate"),
            DevIcon {
                icon: "",
                color: "#9bda5c",

                name: "MATE",
            },
        ),
        (
            OsStr::new("plasma"),
            DevIcon {
                icon: "",
                color: "#1b89f4",

                name: "KDEPlasma",
            },
        ),
        (
            OsStr::new("xfce"),
            DevIcon {
                icon: "",
                color: "#00aadf",

                name: "Xfce",
            },
        ),
    ]);

    let dev_icon_from_wm = HashMap::from_iter([
        (
            OsStr::new("awesomewm"),
            DevIcon {
                icon: "",
                color: "#535d6c",

                name: "awesome",
            },
        ),
        (
            OsStr::new("bspwm"),
            DevIcon {
                icon: "",
                color: "#4f4f4f",

                name: "BSPWM",
            },
        ),
        (
            OsStr::new("dwm"),
            DevIcon {
                icon: "",
                color: "#1177aa",

                name: "dwm",
            },
        ),
        (
            OsStr::new("enlightenment"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "Enlightenment",
            },
        ),
        (
            OsStr::new("fluxbox"),
            DevIcon {
                icon: "",
                color: "#555555",

                name: "Fluxbox",
            },
        ),
        (
            OsStr::new("hyprland"),
            DevIcon {
                icon: "",
                color: "#00aaae",

                name: "Hyprland",
            },
        ),
        (
            OsStr::new("i3"),
            DevIcon {
                icon: "",
                color: "#e8ebee",

                name: "i3",
            },
        ),
        (
            OsStr::new("jwm"),
            DevIcon {
                icon: "",
                color: "#0078cd",

                name: "JWM",
            },
        ),
        (
            OsStr::new("qtile"),
            DevIcon {
                icon: "",
                color: "#ffffff",

                name: "Qtile",
            },
        ),
        (
            OsStr::new("sway"),
            DevIcon {
                icon: "",
                color: "#68751c",

                name: "Sway",
            },
        ),
        (
            OsStr::new("xmonad"),
            DevIcon {
                icon: "",
                color: "#fd4d5d",

                name: "xmonad",
            },
        ),
    ]);

    DevIconsContainer {
        from_file_name: dev_icon_from_filename,
        from_extension: dev_icon_from_extension,
        from_os: dev_icon_from_os,
        from_de: dev_icon_from_de,
        from_wm: dev_icon_from_wm,
    }
});

#[derive(Default)]
pub struct DevIcon {
    pub icon: &'static str,
    pub color: &'static str,
    pub name: &'static str,
}

/// Last update: commit b77921f
impl DevIcon {
    pub fn init(lua: &Lua) -> LuaResult<()> {
        let container = &DEV_ICONS;

        for item in container.from_file_name.iter() {
            let dev_icon = item.1;
            NeoTheme::set_hl(
                lua,
                0,
                &format!("DevIcon{}", dev_icon.name),
                HLOpts {
                    fg: Some(dev_icon.color.to_string()),
                    ..Default::default()
                },
            )?;
        }

        Ok(())
    }

    pub fn get_icon(path: &Path) -> IconResult {
        let container = &DEV_ICONS;

        let dev_icon_from_extension = || -> Option<&DevIcon> {
            let ext = path.extension()?;
            container.from_extension.get(ext)
        };

        if let Some(dev_icon) = container.from_file_name.get(path.as_os_str()) {
            IconResult {
                icon: dev_icon.icon,
                highlight: format!("DevIcon{}", dev_icon.name),
            }
        } else if let Some(dev_icon) = dev_icon_from_extension() {
            IconResult {
                icon: dev_icon.icon,
                highlight: format!("DevIcon{}", dev_icon.name),
            }
        } else if let Some(dev_icon) = container.from_os.get(path.as_os_str()) {
            IconResult {
                icon: dev_icon.icon,
                highlight: format!("DevIcon{}", dev_icon.name),
            }
        } else if let Some(dev_icon) = container.from_de.get(path.as_os_str()) {
            IconResult {
                icon: dev_icon.icon,
                highlight: format!("DevIcon{}", dev_icon.name),
            }
        } else if let Some(dev_icon) = container.from_wm.get(path.as_os_str()) {
            IconResult {
                icon: dev_icon.icon,
                highlight: format!("DevIcon{}", dev_icon.name),
            }
        } else {
            let dev_icon = container.from_extension.get(OsStr::new("txt")).unwrap();

            IconResult {
                icon: dev_icon.icon,
                highlight: format!("DevIcon{}", dev_icon.name),
            }
        }
    }
}

pub struct IconResult {
    pub icon: &'static str,
    pub highlight: String,
}
