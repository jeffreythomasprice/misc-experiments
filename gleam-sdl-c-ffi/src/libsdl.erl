-module(libsdl).

-export([
    init/0,
    sdl_version_compiled/0,
    sdl_version_linked/0,
    sdl_version_major/1,
    sdl_version_minor/1,
    sdl_version_micro/1,
    sdl_get_revision_compiled/0,
    sdl_get_revision_linked/0,
    sdl_get_error/0,
    sdl_clear_error/0,
    sdl_init/1,
    sdl_init_video/0,
    sdl_quit/0,
    sdl_create_window_and_renderer/4
]).

-nifs([
    sdl_version_compiled/0,
    sdl_version_linked/0,
    sdl_version_major/1,
    sdl_version_minor/1,
    sdl_version_micro/1,
    sdl_get_revision_compiled/0,
    sdl_get_revision_linked/0,
    sdl_get_error/0,
    sdl_clear_error/0,
    sdl_init/1,
    sdl_init_video/0,
    sdl_quit/0,
    sdl_create_window_and_renderer/4
]).

-on_load(init/0).

init() ->
    erlang:load_nif("sdl/bin/libsdl", 0).

sdl_version_compiled() ->
    erlang:nif_error("NIF library not loaded").

sdl_version_linked() ->
    erlang:nif_error("NIF library not loaded").

sdl_version_major(x) ->
    erlang:nif_error("NIF library not loaded").

sdl_version_minor(x) ->
    erlang:nif_error("NIF library not loaded").

sdl_version_micro(x) ->
    erlang:nif_error("NIF library not loaded").

sdl_get_revision_linked() ->
    erlang:nif_error("NIF library not loaded").

sdl_get_revision_compiled() ->
    erlang:nif_error("NIF library not loaded").

sdl_get_error() ->
    erlang:nif_error("NIF library not loaded").

sdl_clear_error() ->
    erlang:nif_error("NIF library not loaded").

sdl_init(flags) ->
    erlang:nif_error("NIF library not loaded").

sdl_init_video() ->
    erlang:nif_error("NIF library not loaded").

sdl_quit() ->
    erlang:nif_error("NIF library not loaded").

sdl_create_window_and_renderer(title, width, height, flags) ->
    erlang:nif_error("NIF library not loaded").
