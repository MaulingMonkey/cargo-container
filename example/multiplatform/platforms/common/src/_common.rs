mod arches;     pub use arches::Arches;
mod config;     pub use config::Config;
mod download;   pub use download::Download;
pub mod exit;
mod package;    pub use package::Package;
mod state;      pub use state::State;

pub extern crate cargo_local_install;
pub extern crate mmrbi;



pub fn exec(tool: impl Tool, suffix: &str) {
    let state = State::get(suffix);
    let state = &state;
    match state.command.as_str() {
        "bench"     => tool.bench   (state),
        "build"     => tool.build   (state),
        "clean"     => tool.clean   (state),
        "doc"       => tool.doc     (state),
        "fetch"     => tool.fetch   (state),
        "generate"  => tool.generate(state),
        "run"       => tool.run     (state),
        "package"   => tool.package (state),
        "setup"     => tool.setup   (state),
        "test"      => tool.test    (state),
        "update"    => tool.update  (state),
        _other      => exit::command_not_implemented(),
    }
}



pub trait Tool {
    fn bench    (&self, _state: &State) { exit::command_not_implemented() }
    fn build    (&self, _state: &State) { exit::command_not_implemented() }
    fn clean    (&self, _state: &State) { exit::command_not_implemented() }
    fn doc      (&self, _state: &State) { exit::command_not_implemented() }
    fn fetch    (&self, _state: &State) { exit::command_not_implemented() }
    fn generate (&self, _state: &State) { exit::command_not_implemented() }
    fn run      (&self, _state: &State) { exit::command_not_implemented() }
    fn package  (&self, _state: &State) { exit::command_not_implemented() }
    fn setup    (&self, _state: &State) { exit::command_not_implemented() }
    fn test     (&self, _state: &State) { exit::command_not_implemented() }
    fn update   (&self, _state: &State) { exit::command_not_implemented() }
}
