-module(libexperiment).

-export([
    init/0,
    hello/0,
    foobar/2
]).

-nifs([
    hello/0,
    foobar/2
]).

-on_load(init/0).

init() ->
    erlang:load_nif("priv/libexperimentnif", 0).

% TODO no hello
hello() ->
    erlang:nif_error("NIF library not loaded").

foobar(s, x) ->
    erlang:nif_error("NIF library not loaded").
