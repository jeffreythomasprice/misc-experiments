-module(rust_lib).
-export([add_ints/2]).
-nifs([add_ints/2]).
-on_load(init/0).

init() ->
    ok = erlang:load_nif("priv/librust_lib", 0).

add_ints(a, b) ->
    exit(nif_library_not_loaded).
