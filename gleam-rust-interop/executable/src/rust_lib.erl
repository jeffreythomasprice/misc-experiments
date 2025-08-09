-module(rust_lib).
-export([foo/0]).
-nifs([foo/0]).
-on_load(init/0).

init() ->
    ok = erlang:load_nif("priv/rust_lib", 0).

foo() ->
    exit(nif_library_not_loaded).
