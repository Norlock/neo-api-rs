use mlua::prelude::LuaResult;
use mlua::Lua;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use tokio::fs;
use tokio::io::{self, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::diffuser::{Diffuse, ExecuteChain};
use crate::web_devicons::icons_default::DevIcon;
use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, CmdOpts, FileTypeMatch, HLOpts, Mode, NeoApi,
    NeoBuffer, NeoPopup, NeoTheme, NeoWindow, OpenIn, PopupBorder, PopupRelative, PopupSize,
    PopupStyle, TextType, RTM,
};

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";
const AUCMD_GRP: &str = "neo-fuzzy";

struct LineOut {
    text: String,
    icon: String,
    hl_group: String,
}

struct FuzzyContainer {
    all_lines: RwLock<String>,
    lines_out: RwLock<Vec<LineOut>>,
    fuzzy: RwLock<Option<NeoFuzzy>>,
    search_state: RwLock<SearchState>,
    preview: RwLock<Vec<String>>,
}

pub trait FuzzyConfig: Send + Sync {
    fn cwd(&self, lua: &Lua) -> PathBuf;
    fn search_type(&self) -> FuzzySearch;
    fn on_enter(&self, lua: &Lua, open_in: OpenIn, item: PathBuf);
}

impl std::fmt::Debug for dyn FuzzyConfig {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
struct SearchState {
    last_search: String,
    file_path: String,
    update_out: bool,
    update_preview: bool,
    has_searched: bool,
}

static CONTAINER: Lazy<FuzzyContainer> = Lazy::new(|| FuzzyContainer {
    all_lines: RwLock::new(String::new()),
    lines_out: RwLock::new(vec![]),
    fuzzy: RwLock::new(None),
    search_state: RwLock::new(SearchState {
        update_out: false,
        update_preview: false,
        has_searched: false,
        last_search: "".to_string(),
        file_path: "".to_string(),
    }),
    preview: RwLock::new(Vec::new()),
});

#[derive(Debug)]
pub struct NeoFuzzy {
    pub pop_cmd: NeoPopup,
    pub pop_out: NeoPopup,
    pub pop_preview: NeoPopup,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub cmd: String,
    pub selected_idx: usize,
    pub ns_id: u32,
    pub config: Box<dyn FuzzyConfig>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FuzzySearch {
    File,
    Directory,
}

pub enum Move {
    Up,
    Down,
}

impl NeoFuzzy {
    pub fn add_hl_groups(lua: &Lua) -> LuaResult<()> {
        NeoTheme::set_hl(
            lua,
            0,
            GRP_FUZZY_LETTER,
            HLOpts {
                fg: Some("#39E75F".to_string()),
                bold: true,
                ..Default::default()
            },
        )?;

        NeoTheme::set_hl(
            lua,
            0,
            GRP_FUZZY_SELECT,
            HLOpts {
                fg: Some("#CEFAD0".to_string()),
                ..Default::default()
            },
        )?;

        Ok(())
    }

    pub fn add_keymaps(&self, lua: &Lua) -> LuaResult<()> {
        let buf = self.pop_cmd.buf;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Up>",
            lua.create_async_function(|lua, ()| move_selection(lua, Move::Up))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Down>",
            lua.create_async_function(|lua, ()| move_selection(lua, Move::Down))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Esc>",
            lua.create_function(close_fuzzy)?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<C-t>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::Tab))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<C-s>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::HSplit))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<C-v>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::VSplit))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Enter>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::Buffer))?,
        )
    }

    pub async fn files_or_directories(lua: &Lua, config: Box<dyn FuzzyConfig>) -> LuaResult<()> {
        Self::add_hl_groups(lua)?;

        let ns_id = NeoTheme::create_namespace(lua, "NeoFuzzy")?;

        NeoTheme::set_hl_ns(lua, ns_id)?;

        let ui = &NeoApi::list_uis(lua)?[0];

        let pop_cmd_row = ui.height - 6;
        let pop_cmd_col = 4;
        let pop_cmd_height = 1;
        let pop_cmd_width = if ui.width % 2 == 0 {
            ui.width - 8
        } else {
            ui.width - 9
        };

        let out_preview_height = pop_cmd_row - 4;
        let out_preview_row = 2;

        let out_width = pop_cmd_width / 2;
        let out_col = pop_cmd_col;
        let preview_width = out_width - 2;
        let preview_col = pop_cmd_col + out_width + 2;

        let pop_cmd = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            true,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(pop_cmd_width)),
                height: Some(PopupSize::Fixed(pop_cmd_height)),
                row: Some(PopupSize::Fixed(pop_cmd_row)),
                col: Some(PopupSize::Fixed(pop_cmd_col)),
                relative: PopupRelative::Editor,
                border: PopupBorder::Single,
                style: Some(PopupStyle::Minimal),
                title: Some(TextType::String(" Search query ".to_string())),
                ..Default::default()
            },
        )?;

        let pop_out = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            false,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(out_width)),
                height: Some(PopupSize::Fixed(out_preview_height)),
                row: Some(PopupSize::Fixed(out_preview_row)),
                col: Some(PopupSize::Fixed(out_col)),
                relative: PopupRelative::Editor,
                border: crate::PopupBorder::Single,
                focusable: Some(false),
                style: Some(PopupStyle::Minimal),
                ..Default::default()
            },
        )?;

        pop_out.win.set_option_value(lua, "cursorline", true)?;

        let pop_preview = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            false,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(preview_width)),
                height: Some(PopupSize::Fixed(out_preview_height)),
                row: Some(PopupSize::Fixed(out_preview_row)),
                col: Some(PopupSize::Fixed(preview_col)),
                relative: PopupRelative::Editor,
                border: crate::PopupBorder::Single,
                focusable: Some(false),
                style: Some(PopupStyle::Minimal),
                noautocmd: true,
                ..Default::default()
            },
        )?;

        pop_cmd.buf.set_current(lua)?;
        NeoApi::set_insert_mode(lua, true)?;

        let group = NeoApi::create_augroup(lua, AUCMD_GRP, false)?;

        let callback = lua.create_async_function(aucmd_text_changed)?;

        NeoApi::create_autocmd(
            lua,
            &[AutoCmdEvent::TextChangedI],
            crate::AutoCmdOpts {
                callback,
                buffer: Some(pop_cmd.buf.id()),
                group: Some(AutoCmdGroup::Integer(group)),
                pattern: vec![],
                once: false,
                desc: None,
            },
        )?;

        NeoApi::create_autocmd(
            lua,
            &[AutoCmdEvent::BufLeave],
            crate::AutoCmdOpts {
                callback: lua.create_async_function(aucmd_close_fuzzy)?,
                buffer: Some(pop_cmd.buf.id()),
                group: Some(AutoCmdGroup::Integer(group)),
                pattern: vec![],
                once: true,
                desc: None,
            },
        )?;

        let cmd = "fd".to_string();
        let cwd = config.cwd(lua);

        let args = match config.search_type() {
            FuzzySearch::Directory => {
                vec!["--type".to_string(), "directory".to_string()]
            }
            FuzzySearch::File => {
                vec!["--type".to_string(), "file".to_string()]
            }
        };

        let fuzzy = NeoFuzzy {
            pop_cmd,
            pop_out,
            pop_preview,
            cwd,
            args,
            cmd,
            selected_idx: 0,
            ns_id,
            config,
        };

        fuzzy.add_keymaps(lua)?;

        let cwd = fuzzy.cwd.clone();
        let cmd = fuzzy.cmd.clone();
        let args = fuzzy.args.clone();
        let search_type = fuzzy.config.search_type();

        *CONTAINER.all_lines.write().await = String::new();
        CONTAINER.search_state.write().await.has_searched = false;

        Diffuse::queue(Box::new(ExecSearch {
            search_query: "".to_string(),
            cwd,
            cmd,
            args,
            search_type,
        }))
        .await;

        Diffuse::start().await;
        //RTM.spawn(async move {
        //let _ = exec_search(&cwd, cmd, args, "", search_type).await;
        //let _ = prepare_preview(&cwd, 0).await;
        //});

        let mut container = CONTAINER.fuzzy.write().await;
        *container = Some(fuzzy);
        drop(container);

        let interval = lua.create_async_function(interval_write_out)?;

        NeoApi::start_interval(lua, "fuzzy", 32, interval)?;

        Ok(())
    }

    pub fn fuzzy_grep(_cwd: &Path, _text: String) {
        //
    }

    fn add_preview_highlight(&self, lua: &Lua, preview: &[String]) -> LuaResult<()> {
        self.pop_preview
            .buf
            .clear_namespace(lua, self.ns_id as i32, 0, -1)?;

        for (i, item_name) in preview.iter().enumerate() {
            if item_name.ends_with('/') {
                self.pop_preview.buf.add_highlight(
                    lua,
                    self.ns_id as i32,
                    "Directory",
                    i,
                    0,
                    -1,
                )?;
            }

            if item_name.starts_with("> Empty directory")
                || item_name.starts_with("> File is a binary")
            {
                self.pop_preview
                    .buf
                    .add_highlight(lua, self.ns_id as i32, "Comment", i, 0, -1)?;
            }
        }

        Ok(())
    }

    fn add_out_highlight(&self, lua: &Lua) -> LuaResult<()> {
        self.pop_out
            .buf
            .clear_namespace(lua, self.ns_id as i32, 0, -1)?;

        self.pop_out.buf.add_highlight(
            lua,
            self.ns_id as i32,
            "NeoFuzzySelect",
            self.selected_idx,
            0,
            -1,
        )?;

        if let Ok(lines) = CONTAINER.lines_out.try_read() {
            let lines = if 300 <= lines.len() {
                &lines[..300]
            } else {
                &lines
            };

            for (i, line) in lines.iter().enumerate() {
                self.pop_out
                    .buf
                    .add_highlight(lua, self.ns_id as i32, &line.hl_group, i, 0, 2)?;
            }
        }

        Ok(())
    }
}

pub async fn open_item(lua: &Lua, open_in: OpenIn) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;

    let fuzzy = fuzzy.as_ref().unwrap();
    let cached_lines = CONTAINER.lines_out.read().await;
    let selected = fuzzy
        .cwd
        .join(cached_lines[fuzzy.selected_idx].text.as_str());
    fuzzy.pop_cmd.win.close(lua, false)?;

    fuzzy.config.on_enter(lua, open_in, selected);

    Ok(())
}

struct SortLines {
    lines_out: Vec<LineOut>,
    search_query: String,
}

impl ExecuteChain for SortLines {
    fn try_execute(
        self: Box<Self>,
    ) -> Pin<Box<dyn std::future::Future<Output = crate::diffuser::ChainLink> + Send>> {
        Box::pin(async move {
            let lines_out = CONTAINER.lines_out.try_write();
            let interval = CONTAINER.search_state.try_write();

            if lines_out.is_err() || interval.is_err() {
                let ret: Box<dyn ExecuteChain> = self;
                return Some(ret);
            }

            let mut lines_out = lines_out.unwrap();
            let mut interval = interval.unwrap();

            *lines_out = self.lines_out;

            interval.last_search = self.search_query;
            interval.update_out = true;
            interval.has_searched = true;

            None
        })

        //fn try_execute(self: Box<Self>) -> Option<Box<dyn ExecuteChain>> {
    }
}

fn sort_lines(
    lines: &str,
    search_query: String,
    search_type: FuzzySearch,
) -> Pin<Box<dyn std::future::Future<Output = crate::diffuser::ChainLink> + Send>> {
    let mut new_lines = Vec::new();

    for line in lines.lines() {
        let score = levenshtein(&search_query, line);
        new_lines.push((score, line.to_string()));
    }

    new_lines.sort_by_key(|k| k.0);

    let mut lines_out = Vec::new();

    for (_k, v) in new_lines.into_iter() {
        if search_type == FuzzySearch::Directory {
            lines_out.push(LineOut {
                text: v,
                icon: "".to_string(),
                hl_group: "Directory".to_string(),
            });
        } else {
            let path = PathBuf::from(&v);
            if let Some(dev_icon) = DevIcon::get_icon(&path) {
                lines_out.push(LineOut {
                    text: v,
                    icon: dev_icon.icon.to_string(),
                    hl_group: dev_icon.highlight.to_string(),
                });
            } else {
                lines_out.push(LineOut {
                    text: v,
                    icon: "".to_string(),
                    hl_group: "".to_string(),
                });
            }
        }
    }

    let sort_lines = Box::new(SortLines {
        lines_out,
        search_query,
    });

    sort_lines.try_execute()
}

struct ExecSearch {
    cwd: PathBuf,
    cmd: String,
    args: Vec<String>,
    search_query: String,
    search_type: FuzzySearch,
}

impl ExecuteChain for ExecSearch {
    fn try_execute(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::diffuser::ChainLink> + Send>>
    {
        Box::pin(async move {
            let sanitized = self.search_query.replace('/', "\\/").replace('.', "\\.");

            let search_state = CONTAINER.search_state.try_read();

            if search_state.is_err() {
                let ret: Box<dyn ExecuteChain> = self;
                return Some(ret);
            }

            let search_state = search_state.unwrap();

            if !search_state.has_searched {
                //return Some(self);
                let out = Command::new(&self.cmd)
                    .current_dir(&self.cwd)
                    .args(&self.args)
                    .output()
                    .await
                    .unwrap();

                if out.status.success() {
                    if let Ok(mut lines) = CONTAINER.all_lines.try_write() {
                        *lines = String::from_utf8_lossy(&out.stdout).to_string();
                        return sort_lines(&lines, "".to_string(), self.search_type).await;
                    } else {
                        return Some(self);
                    }
                } else {
                    panic!("Unsuccessfull search query");
                }
            } else if self.search_query.is_empty() {
                if let Ok(lines) = CONTAINER.all_lines.try_read() {
                    return sort_lines(&lines, "".to_string(), self.search_type).await;
                } else {
                    return Some(self);
                }
            } else {
                let mut regex = String::from(".*");

                for char in sanitized.chars() {
                    if char.is_lowercase() {
                        regex.push_str(&format!("[{}{}]", char.to_uppercase(), char));
                    } else {
                        regex.push(char);
                    }

                    if char != '\\' {
                        regex.push_str(".*");
                    }
                }

                let mut rg_proc = tokio::process::Command::new("rg")
                    .args(["--regexp", &regex])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                let mut stdin = rg_proc.stdin.take().unwrap();

                let search_qry_len = self.search_query.len();

                RTM.spawn(async move {
                    let meta = CONTAINER.search_state.read().await;
                    let lines = CONTAINER.lines_out.read().await;

                    if meta.last_search.len() < search_qry_len {
                        for line in lines.iter() {
                            let line = format!("{}\n", line.text);
                            let _ = stdin.write_all(line.as_bytes()).await;
                        }
                    } else {
                        if let Ok(lines) = CONTAINER.all_lines.try_read() {
                            stdin.write_all(lines.as_bytes()).await.unwrap();
                        }
                    }

                    let _ = stdin.flush().await;
                });
                //RTM.block_on(async move {
                //});

                //RTM.handle().block_on(async {
                let out = rg_proc.wait_with_output().await.unwrap();

                if out.status.success() {
                    let lines = String::from_utf8_lossy(&out.stdout);
                    return sort_lines(&lines, self.search_query, self.search_type).await;
                } else {
                    return Some(self);
                }
                //return Some(self);
            }
        })
    }
}
//fn try_execute(self: Box<Self>) -> Option<Box<dyn ExecuteChain>> {

//async fn exec_search(
//cwd: &Path,
//cmd: String,
//args: Vec<String>,
//search_query: &str,
//search_type: FuzzySearch,
//) -> io::Result<()> {
//let sanitized = search_query.replace('/', "\\/").replace('.', "\\.");

//let has_searched = CONTAINER.search_state.read().await.has_searched;

//if !has_searched {
//let out = Command::new(cmd)
//.current_dir(cwd)
//.args(args)
//.output()
//.await?;

//if out.status.success() {
//let mut lines = CONTAINER.all_lines.write().await;
//*lines = String::from_utf8_lossy(&out.stdout).to_string();
//sort_lines(&lines, "", search_type).await;
//}
//} else if search_query.is_empty() {
//let lines = CONTAINER.all_lines.read().await;
//sort_lines(&lines, "", search_type).await;
//} else {
//let mut regex = String::from(".*");

//for char in sanitized.chars() {
//if char.is_lowercase() {
//regex.push_str(&format!("[{}{}]", char.to_uppercase(), char));
//} else {
//regex.push(char);
//}

//if char != '\\' {
//regex.push_str(".*");
//}
//}

//let mut rg_proc = Command::new("rg")
//.args(["--regexp", &regex])
//.stdin(Stdio::piped())
//.stdout(Stdio::piped())
//.stderr(Stdio::piped())
//.spawn()?;

//let mut stdin = rg_proc.stdin.take().unwrap();

//let search_qry_len = search_query.len();

//tokio::spawn(async move {
//let meta = CONTAINER.search_state.read().await;

//if meta.last_search.len() < search_qry_len {
//let lines = CONTAINER.lines_out.read().await;

//for line in lines.iter() {
//let line = format!("{}\n", line.text);
//let _ = stdin.write_all(line.as_bytes()).await;
//}
//} else {
//let lines = CONTAINER.all_lines.read().await;
//stdin.write_all(lines.as_bytes()).await.unwrap();
//}

//let _ = stdin.flush().await;
//});

//let out = rg_proc.wait_with_output().await?;

//if out.status.success() {
//let lines = String::from_utf8_lossy(&out.stdout);
//sort_lines(&lines, search_query, search_type).await;
//}
//}

//Ok(())
//}

async fn preview_file(path: &Path) -> io::Result<()> {
    let file_path;

    if is_binary(path) {
        file_path = "text".to_string();

        let mut preview = CONTAINER.preview.write().await;
        *preview = vec!["> File is a binary".to_string()];
    } else {
        file_path = path.to_string_lossy().to_string();

        let mut lines = vec![];
        let file = fs::read_to_string(path).await?;

        for line in file.lines() {
            lines.push(line.to_string());
        }

        let mut preview = CONTAINER.preview.write().await;
        *preview = lines
    }

    let mut interval_state = CONTAINER.search_state.write().await;
    interval_state.file_path = file_path;
    interval_state.update_preview = true;

    Ok(())
}

async fn preview_directory(path: &Path) -> io::Result<()> {
    let mut items = Vec::new();

    let mut dir = fs::read_dir(path).await?;

    while let Some(item) = dir.next_entry().await? {
        if let Ok(file_type) = item.file_type().await {
            let name = item.file_name().to_string_lossy().to_string();

            if file_type.is_dir() {
                items.push(format!("{name}/"));
            } else {
                items.push(name);
            }
        }
    }

    items.sort_by(|a, b| {
        if a.ends_with('/') == b.ends_with('/') {
            a.cmp(b)
        } else if a.ends_with('/') {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    if items.is_empty() {
        items.push("> Empty directory".to_string());
    }

    *CONTAINER.preview.write().await = items;
    CONTAINER.search_state.write().await.update_preview = true;

    Ok(())
}

async fn prepare_preview(cwd: &Path, selected_idx: usize) -> io::Result<()> {
    let path: PathBuf = {
        let lines_out = CONTAINER.lines_out.read().await;

        if lines_out.is_empty() {
            CONTAINER.preview.write().await.clear();
            CONTAINER.search_state.write().await.update_preview = true;
            return Ok(());
        }

        cwd.join(lines_out[selected_idx].text.as_str())
    };

    if path.is_dir() {
        preview_directory(&path).await?;
    } else if path.is_file() {
        preview_file(&path).await?;
    }

    Ok(())
}

async fn interval_write_out(lua: &Lua, _: ()) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.try_read();

    if fuzzy.is_err() {
        return Ok(());
    }

    let fuzzy = fuzzy.unwrap();

    if let Some(fuzzy) = fuzzy.as_ref() {
        let interval = CONTAINER.search_state.try_write();
        let lines = CONTAINER.lines_out.try_read();

        if interval.is_err() || lines.is_err() {
            return Ok(());
        }

        let mut interval = interval.unwrap();
        let lines = lines.unwrap();

        if interval.update_out {
            interval.update_out = false;

            fuzzy.add_out_highlight(lua)?;
            let lines = if 300 <= lines.len() {
                &lines[..300]
            } else {
                &lines
            };

            if FuzzySearch::File == fuzzy.config.search_type() {
                let mut icon_lines = Vec::new();

                for line in lines {
                    icon_lines.push(format!(" {} {}", line.icon, line.text));
                }

                fuzzy
                    .pop_out
                    .buf
                    .set_lines(lua, 0, -1, false, &icon_lines)?;
            } else {
                let lines: Vec<_> = lines
                    .iter()
                    .map(|line| format!(" {} {}", line.icon, line.text))
                    .collect();

                fuzzy.pop_out.buf.set_lines(lua, 0, -1, false, &lines)?;
            }
        }

        let preview = CONTAINER.preview.try_read();
        if interval.update_preview && preview.is_ok() {
            interval.update_preview = false;
            let file_path = interval.file_path.to_string();

            fuzzy.add_out_highlight(lua)?;

            let buf = &fuzzy.pop_preview.buf;

            let preview = preview.unwrap();
            buf.set_lines(lua, 0, -1, false, &preview)?;
            fuzzy.add_preview_highlight(lua, &preview)?;

            if fuzzy.config.search_type() == FuzzySearch::File {
                let ft = NeoApi::filetype_match(
                    lua,
                    FileTypeMatch {
                        filename: Some(file_path),
                        contents: None,
                        buf: Some(buf.id()),
                    },
                )?;

                buf.stop_treesitter(lua)?;

                if let Some(ft) = ft {
                    let lang = buf.get_treesitter_lang(lua, &ft)?;

                    if let Some(lang) = lang {
                        buf.start_treesitter(lua, &lang)?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn move_selection(lua: &Lua, move_sel: Move) -> LuaResult<()> {
    let mut fuzzy = CONTAINER.fuzzy.write().await;

    if let Some(fuzzy) = fuzzy.as_mut() {
        let len = fuzzy.pop_out.buf.line_count(lua)?;
        let mut jump_line = false;

        match move_sel {
            Move::Up => {
                if 0 < fuzzy.selected_idx {
                    fuzzy.selected_idx -= 1;
                    jump_line = true;
                }
            }
            Move::Down => {
                if fuzzy.selected_idx + 1 < len {
                    fuzzy.selected_idx += 1;
                    jump_line = true;
                }
            }
        }

        if jump_line {
            let sel_idx = fuzzy.selected_idx;
            let cwd = fuzzy.cwd.clone();

            fuzzy.pop_out.win.call(
                lua,
                lua.create_function(move |lua, _: ()| {
                    NeoApi::cmd(
                        lua,
                        CmdOpts {
                            cmd: "normal",
                            bang: true,
                            args: &[&format!("{}G", sel_idx + 1)],
                        },
                    )
                })?,
            )?;

            RTM.spawn(async move {
                let _ = prepare_preview(&cwd, sel_idx).await;
            });
        }
    }

    Ok(())
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

async fn aucmd_close_fuzzy(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let container = CONTAINER.fuzzy.read().await;

    if let Some(fuzzy) = container.as_ref() {
        fuzzy.pop_out.win.close(lua, false)?;
        fuzzy.pop_cmd.win.close(lua, false)?;
        fuzzy.pop_preview.win.close(lua, false)?;
    }

    Diffuse::stop().await;

    NeoApi::del_augroup_by_name(lua, AUCMD_GRP)?;
    NeoApi::stop_interval(lua, "fuzzy")?;
    NeoApi::set_insert_mode(lua, false)
}

async fn aucmd_text_changed(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let search_query = NeoApi::get_current_line(lua)?;
    let mut fuzzy = CONTAINER.fuzzy.write().await;

    if let Some(fuzzy) = fuzzy.as_mut() {
        fuzzy.selected_idx = 0;

        let cwd = fuzzy.cwd.clone();
        let cmd = fuzzy.cmd.clone();
        let args = fuzzy.args.clone();
        let search_type = fuzzy.config.search_type();
        let sel_idx = fuzzy.selected_idx;

        fuzzy.pop_out.win.call(
            lua,
            lua.create_function(move |lua, _: ()| {
                NeoApi::cmd(
                    lua,
                    CmdOpts {
                        cmd: "normal",
                        bang: true,
                        args: &[&format!("{}G", sel_idx + 1)],
                    },
                )
            })?,
        )?;

        let exec_search = Box::new(ExecSearch {
            search_query,
            cwd,
            cmd,
            args,
            search_type,
        });

        RTM.spawn(Diffuse::queue(exec_search));
        //RTM.spawn(async move {
        //let _ = exec_search(&cwd, cmd, args, &search_query, search_type).await;
        //let _ = prepare_preview(&cwd, 0).await;
        //});
    }

    Ok(())
}

// TODO split on path
pub fn levenshtein(a: &str, b: &str) -> usize {
    let mut result = 0;

    /* Shortcut optimizations / degenerate cases. */
    if a == b {
        return result;
    }

    let length_a = a.len();
    let length_b = b.len();

    if length_a == 0 {
        return length_b;
    } else if length_b == 0 {
        return length_a;
    }

    /* Initialize the vector.
     *
     * This is why it’s fast, normally a matrix is used,
     * here we use a single vector. */
    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    /* Loop. */
    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    result
}

fn is_binary(file: &Path) -> bool {
    let binaries: HashSet<&str> = [
        "bin", "so", "mkv", "mp4", "blend", "jpg", "png", "jpeg", "webp",
    ]
    .into();

    if let Some(ext) = file.extension() {
        binaries.contains(ext.to_str().unwrap())
    } else {
        false
    }
}
