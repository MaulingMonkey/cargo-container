//! Utilities for managing VS Code integration.

use serde::{Serialize, Serializer, Deserialize, ser::SerializeMap};
use serde_json::{json, Value};

use std::fs;
use std::io::{self};
use std::mem;
use std::path::{Path};
use std::process::Command;
use std::result::Result;



#[cfg(test)]
macro_rules! assert_eq_lines {
    ($actual:expr, $expected:expr) => {{
        let mut actual = $actual.lines();
        let mut expected = $expected.lines();

        let mut line = 0;
        loop {
            line += 1;
            let actual = actual.next();
            let expected = expected.next();
            match (actual, expected) {
                (None,          None            ) => break,
                (Some(actual),  Some(expected)  ) => assert_eq!(actual, expected, "line {}", line),
                (Some(_actual), None            ) => panic!("Actual input was longer than expected"),
                (None,          Some(_expected) ) => panic!("Actual input was shorter than expected"),
            }
        }
    }}
}

fn code_cmd() -> Command {
    if cfg!(windows) {
        // VS Code might be a .cmd script, rust's spawn behavior ignores PATHEXT and so doesn't work for us.
        // https://github.com/rust-lang/rust/issues/37519
        // https://github.com/rust-lang/rust/blob/ac968c466451cb9aafd9e8598ddb396ed0e6fe31/src/libstd/sys/windows/process.rs#L130-L149
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("code");
        cmd
    } else {
        Command::new("code")
    }
}

/// Is VS Code installed?
/// 
/// Implemented as a `code --version` query.
pub fn is_installed() -> bool {
    Version::get().is_ok()
}

#[test] fn test_is_installed() {
    let _ = is_installed();
}

/// The result of a `code --version` query.
#[derive(Debug)]
pub struct Version {
    /// The human-readable version identifier (e.g. "1.38.0")
    pub version:    String,

    /// A git hash (e.g. "3db7e09f3b61f915d03bbfa58e258d6eee843f35")
    /// This corresponds to e.g. https://github.com/microsoft/vscode/commit/3db7e09f3b61f915d03bbfa58e258d6eee843f35
    pub hash:       String,

    /// The computer architecture code was built for (e.g. "x64")
    pub arch:       String,

    // --version may add additional lines in the future
    #[doc(hidden)] _non_exhaustive: (),
}

impl Version {
    /// Check the currently installed VS Code version by running `code --version` and parsing the result.
    pub fn get() -> io::Result<Self> {
        let ioerr = |msg| io::Error::new(io::ErrorKind::Other, msg);
        let mut output = code_cmd().arg("--version").output()?;

        if !output.status.success() {
            return Err(ioerr("`code --version` failed"));
        }

        let output = String::from_utf8(mem::replace(&mut output.stdout, Vec::new()));
        let output = output.map_err(|_| ioerr("`code --version` returned invalid UTF8"))?;
        let mut output = output.lines();

        let version = output.next().ok_or_else(|| ioerr("`code --version` missing version line"))?.to_string();
        let hash    = output.next().ok_or_else(|| ioerr("`code --version` missing hash line"))?.to_string();
        let arch    = output.next().ok_or_else(|| ioerr("`code --version` missing arch line"))?.to_string();

        Ok(Self{
            version,
            hash,
            arch,

            _non_exhaustive: (),
        })
    }
}

#[test] fn test_version_get() {
    match Version::get() {
        Ok(version) => {
            assert!(version.version.starts_with("1."));
            assert_eq!(version.hash.len(), 40);
            if cfg!(target_arch = "x86_64") { assert_eq!(version.arch, "x64"); }
        },
        err => panic!("Version get failed: {:?}", err),
    }
}



/// A `.vscode` directory.
pub struct DotDirectory<'a>(&'a Path);

impl DotDirectory<'_> {
    /// Write to the `.vscode/extensions.json` file.
    pub fn set_extensions(&mut self, extensions: ExtensionsJson<'_>) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&extensions)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Failed to serialize ExtensionsJson to JSON: {:?}", err)))?;
        let path = self.0.join("extensions.json");
        fs::write(path, json)
    }
}

/// Represents a `.vscode/extensions.json` file.
#[derive(Serialize)]
pub struct ExtensionsJson<'a> {
    /// Extensions VS Code should recommend to the user of this workspace.
    pub recommendations: &'a [MarketplaceExtension<'a>],

    /// Extensions VS Code should *not* recommend to the user of this workspace.
    #[serde(rename = "unwantedRecommendations")]
    pub unwanted_recommendations: &'a [MarketplaceExtension<'a>],

    // In case .vscode/extensions.json gains new fields in the future.
    #[serde(skip)]
    _non_exhaustive: (),
}

impl ExtensionsJson<'_> {
    pub fn new() -> Self {
        Self {
            recommendations:            &[][..],
            unwanted_recommendations:   &[][..],
            _non_exhaustive:            (),
        }
    }
}

impl Default for ExtensionsJson<'_> {
    fn default() -> Self { Self::new() }
}



/// A `.vscode/tasks.json` or `.vscode/launch.json` input variable definition.
/// 
/// https://code.visualstudio.com/docs/editor/variables-reference
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum InputVariable {
    PromptString {
        //type:         "promptString"
        id:             String,
        #[serde(skip_serializing_if = "Option::is_none")]   description:    Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]   default:        Option<String>,
    },
    PickString {
        //type:         "pickString"
        id:             String,
        #[serde(skip_serializing_if = "Option::is_none")]   description:    Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]   default:        Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty"  )]   options:        Vec<String>,
    },
    Command {
        //type:         "command"
        id:             String,
        command:        String,
        #[serde(skip_serializing_if = "Value::is_null" )]   args:           Value, // Some commands take strings, others take maps...
    },

    #[doc(hidden)] _NonExhaustive,
}

impl InputVariable {
    /// The "id" field of the input variable.
    pub fn id(&self) -> &str {
        match self {
            InputVariable::PromptString { id, .. } => id,
            InputVariable::PickString   { id, .. } => id,
            InputVariable::Command      { id, .. } => id,
            InputVariable::_NonExhaustive => panic!("InputVariable::_NonExhaustive"),
        }.as_str()
    }

    /// The "type" field of the input variable.
    pub fn ty(&self) -> &str {
        match self {
            InputVariable::PromptString { .. } => "promptString",
            InputVariable::PickString   { .. } => "pickString",
            InputVariable::Command      { .. } => "command",
            InputVariable::_NonExhaustive => panic!("InputVariable::_NonExhaustive"),
        }
    }
}

/// A `.vscode/tasks.json` task to open a url/website such as external documentation, repositories, issue links, etc.
pub struct OpenUrlTask {
    pub group:  TaskGroup,
    pub label:  String,
    pub url:    String,

    #[doc(hidden)] _non_exhaustive: (),
}

impl OpenUrlTask {
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            group:              TaskGroup::None,
            label:              label.into(),
            url:                url.into(),
            _non_exhaustive:    ()
        }
    }
}

impl Task for OpenUrlTask {
    fn to_json(&self) -> Value {
        json!({
            "group":    self.group.to_json(),
            "label":    self.label.clone(),
            "linux":    { "command": format!("xdg-open {}",     self.url) },
            "osx":      { "command": format!("open {}",         self.url) },
            "type":     "shell",
            "windows":  { "command": format!("start \"\" {}",   self.url) },
        })
    }
}

#[test]
fn tasks_json() {
    let mut file = TasksJson200::new();
    file.inputs = vec![
        InputVariable::PromptString {
            id:             "example_prompt_string".to_owned(),
            description:    Some("Example description?".to_owned()),
            default:        Some("option_2".to_owned()),
        },
        InputVariable::PickString {
            id:             "example_pick_string".to_owned(),
            description:    Some("Example description?".to_owned()),
            default:        Some("option_2".to_owned()),
            options:        vec!["option_1".to_owned(), "option_2".to_owned(), "option_3".to_owned()],
        },
        InputVariable::Command {
            id:             "example_command_1".to_owned(),
            command:        "no_args_command".to_owned(),
            args:           Value::Null,
        },
        InputVariable::Command {
            id:             "example_command_2".to_owned(),
            command:        "string_arg_command".to_owned(),
            args:           Value::String("asdf".to_owned()),
        },
    ];
    file.tasks = vec![
        Box::new(OpenUrlTask { group: TaskGroup::BUILD_DEFAULT, ..OpenUrlTask::new("Open github", "https://github.com/MaulingMonkey/cargo-container") }),
        Box::new(OpenUrlTask::new("Open docs.rs", "https://docs.rs/cargo-container")),
    ];


    let actual = serde_json::to_string_pretty(&file).unwrap().replace("  ", "    ").replace("\n", "\n    ");
    let expected = r#"{
        "version": "2.0.0",
        "inputs": [
            {
                "type": "promptString",
                "id": "example_prompt_string",
                "description": "Example description?",
                "default": "option_2"
            },
            {
                "type": "pickString",
                "id": "example_pick_string",
                "description": "Example description?",
                "default": "option_2",
                "options": [
                    "option_1",
                    "option_2",
                    "option_3"
                ]
            },
            {
                "type": "command",
                "id": "example_command_1",
                "command": "no_args_command"
            },
            {
                "type": "command",
                "id": "example_command_2",
                "command": "string_arg_command",
                "args": "asdf"
            }
        ],
        "presentation": {
            "clear": true
        },
        "problemMatcher": [
            "$rustc"
        ],
        "type": "shell",
        "options": {
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_BACKTRACE": "1"
            }
        },
        "tasks": [
            {
                "group": {
                    "isDefault": true,
                    "kind": "build"
                },
                "label": "Open github",
                "linux": {
                    "command": "xdg-open https://github.com/MaulingMonkey/cargo-container"
                },
                "osx": {
                    "command": "open https://github.com/MaulingMonkey/cargo-container"
                },
                "type": "shell",
                "windows": {
                    "command": "start \"\" https://github.com/MaulingMonkey/cargo-container"
                }
            },
            {
                "group": "none",
                "label": "Open docs.rs",
                "linux": {
                    "command": "xdg-open https://docs.rs/cargo-container"
                },
                "osx": {
                    "command": "open https://docs.rs/cargo-container"
                },
                "type": "shell",
                "windows": {
                    "command": "start \"\" https://docs.rs/cargo-container"
                }
            }
        ]
    }"#;
    assert_eq_lines!(actual, expected);
}

/// Represents an opaque `.vscode/tasks.json` task.
pub trait Task {
    fn to_json(&self) -> Value;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskGroup {
    None,
    Build   { default: bool },
    Test    { default: bool },

    #[doc(hidden)]
    _NonExhaustive,
}

impl TaskGroup {
    pub const NONE          : TaskGroup = TaskGroup::None;
    pub const BUILD         : TaskGroup = TaskGroup::Build { default: false };
    pub const BUILD_DEFAULT : TaskGroup = TaskGroup::Build { default: true  };
    pub const TEST          : TaskGroup = TaskGroup::Test  { default: false };
    pub const TEST_DEFAULT  : TaskGroup = TaskGroup::Test  { default: true  };

    pub fn to_json(&self) -> Value {
        match *self {
            TaskGroup::None                 => json!("none"),
            TaskGroup::Build { default }    => if default { json!({ "kind": "build", "isDefault": true }) } else { json!("build") },
            TaskGroup::Test  { default }    => if default { json!({ "kind": "test",  "isDefault": true }) } else { json!("test")  },
            TaskGroup::_NonExhaustive       => panic!("TaskGroup::_NonExhaustive"),
        }
    }
}

/// Represents a `.vscode/tasks.json` file in "version": "2.0.0" format.
pub struct TasksJson200 {
    //version:  "2.0.0"
    pub inputs: Vec<InputVariable>,
    pub tasks:  Vec<Box<dyn Task>>,

    #[doc(hidden)] _non_exhaustive: (),
}

impl TasksJson200 {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            tasks:  Vec::new(),
            _non_exhaustive: ()
        }
    }
}

impl Serialize for TasksJson200 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("version",          "2.0.0")?;
        if !self.inputs.is_empty() {
            map.serialize_entry("inputs",           &self.inputs)?;
        }
        map.serialize_entry("presentation",     &json!({ "clear": true }))?;
        map.serialize_entry("problemMatcher",   &json!(["$rustc"]))?;
        map.serialize_entry("type",             "shell")?;
        map.serialize_entry("options", &json!({
            "cwd": "${workspaceFolder}",
            "env": { "RUST_BACKTRACE": "1" },
        }))?;
        if !self.tasks.is_empty() {
            map.serialize_entry("tasks", &self.tasks.iter().map(|task| task.to_json()).collect::<Vec<Value>>())?;
        }
        map.end()
    }
}



// SECURITY NOTE: To avoid shell or URL injection attacks, the contents of the MarketplaceExtension should be
// limited to alphanumeric characters and limited punctuation (".-" ?).  This is currently semi-enforced by having the
// fields private, and simply constructing a limited set of known extension identifiers in the first place.
//
// This is probably a bit paranoid and pointless - since installing extensions will allow them to run whatever commands
// they want *anyways* - but I'd rather start with as secure of a design as possible, and later weaken it, than try to
// retrofit security into this later.

/// A [VS Code marketplace](https://marketplace.visualstudio.com/) extension identifier.
/// 
/// ```text
/// Given:  https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools
/// MarketplaceExtension refers to:                             ^^^^^^^^^^^^^^^^^^
/// ```
pub struct MarketplaceExtension<'a>(&'a str);

impl MarketplaceExtension<'_> {
    /// C/C++ for Visual Studio Code
    /// 
    /// MarketplaceExtension for native debugging using either GDB, LLDB, or a native Microsoft debug engine.
    /// * [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools)
    /// * [Repository](https://github.com/Microsoft/vscode-cpptools)
    pub const MS_VSCODE_CPPTOOLS : MarketplaceExtension<'static> = MarketplaceExtension("ms-vscode.cpptools");

    /// Rust support for Visual Studio Code
    /// 
    /// Provides syntax highlighting, intellisense, default build tasks.
    /// * [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust)
    /// * [Repository](https://github.com/rust-lang/rls-vscode)
    pub const RUST_LANG_RUST : MarketplaceExtension<'static> = MarketplaceExtension("rust-lang.rust");

    /// VS Code - Debugger for Chrome
    /// 
    /// Allows you to launch chrome with dev flags and debug/step-through WASM for wasm32-unknown-unknown projects.
    /// * [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=msjsdiag.debugger-for-chrome)
    /// * [Repository](https://github.com/Microsoft/vscode-chrome-debug)
    pub const VSCODE_CHROME_DEBUG : MarketplaceExtension<'static> = MarketplaceExtension("vscode-chrome-debug");

    /// Get the marketplace URL for the extension.
    pub fn marketplace_url(&self) -> String {
        format!("https://marketplace.visualstudio.com/items?itemName={}", self.0)
    }

    /// Install the extension, if not already installed.
    /// 
    /// # SECURITY WARNING
    /// 
    /// VS Code extensions are not sandboxed, and can run arbitrary code.
    /// While Microsoft [does respond to security issues](https://code.visualstudio.com/blogs/2018/11/26/event-stream),
    /// they don't - to my knowledge - proactively review and search for malicious code.
    pub fn install_possibly_malicious_code_from_the_internet(&self) -> io::Result<()> {
        println!("code --install-extension {} --force", self.0);
        let status = code_cmd().args(&["--install-extension", self.0, "--force"]).status()?;
        match status.code() {
            Some(0) => Ok(()),
            _ => Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {:?}: {:?}", self.0, status))),
        }
    }

    /// Disable the extension
    pub fn disable(&self) -> io::Result<()> {
        println!("code --disable-extension {}", self.0);
        let status = code_cmd().args(&["--disable-extension", self.0]).status()?;
        match status.code() {
            Some(0) => Ok(()),
            _ => Err(io::Error::new(io::ErrorKind::Other, format!("Failed to disable {:?}: {:?}", self.0, status))),
        }
    }
}

impl Serialize for MarketplaceExtension<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.0)
    }
}
