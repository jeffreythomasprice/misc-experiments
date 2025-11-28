import gleam/erlang/charlist

pub type InitFlags

@external(erlang, "libsdl", "sdl_version_compiled")
pub fn version_compiled() -> Int

@external(erlang, "libsdl", "sdl_version_linked")
pub fn version_linked() -> Int

@external(erlang, "libsdl", "sdl_version_major")
pub fn version_major(x: Int) -> Int

@external(erlang, "libsdl", "sdl_version_minor")
pub fn version_minor(x: Int) -> Int

@external(erlang, "libsdl", "sdl_version_micro")
pub fn version_micro(x: Int) -> Int

@external(erlang, "libsdl", "sdl_get_revision_compiled")
pub fn get_revision_compiled() -> charlist.Charlist

@external(erlang, "libsdl", "sdl_get_revision_linked")
pub fn get_revision_linked() -> charlist.Charlist

@external(erlang, "libsdl", "sdl_get_error")
pub fn get_error() -> charlist.Charlist

@external(erlang, "libsdl", "sdl_clear_error")
pub fn clear_error() -> Nil

@external(erlang, "libsdl", "sdl_init")
pub fn init(flags: InitFlags) -> Int

@external(erlang, "libsdl", "sdl_init_video")
pub fn init_video() -> InitFlags

@external(erlang, "libsdl", "sdl_quit")
pub fn quit() -> InitFlags
