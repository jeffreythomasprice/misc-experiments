-module(rust_lib).
-export([
    add_ints/2,
    new_data/1,
    data_get/1,
    data_increment/2
]).
-nifs([
    add_ints/2,
    new_data/1,
    data_get/1,
    data_increment/2
]).
-on_load(init/0).

init() ->
    ok = erlang:load_nif("priv/librust_lib", 0).

add_ints(a, b) ->
    exit(nif_library_not_loaded).

new_data(a) ->
    exit(nif_library_not_loaded).

data_get(a) ->
    exit(nif_library_not_loaded).

data_increment(a, b) ->
    exit(nif_library_not_loaded).
