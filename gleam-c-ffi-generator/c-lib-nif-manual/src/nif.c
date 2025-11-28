#include <erl_nif.h>

#include "lib.h"

// TODO no hello
static ERL_NIF_TERM hello(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_string(env, "Hello, world!", ERL_NIF_LATIN1);
}

static ERL_NIF_TERM foobar_nif(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
	unsigned int s_len;
	enif_get_string_length(env, argv[0], &s_len, ERL_NIF_LATIN1);
	char *s = malloc(s_len + 1);
	if (!enif_get_string(env, argv[0], s, s_len + 1, ERL_NIF_LATIN1)) {
		return enif_make_badarg(env);
	}
	
	int x;
	if (!enif_get_int(env, argv[1], &x)) {
		return enif_make_badarg(env);
	}

	// TODO handle error response for bad input, return an Ok or Error
    int result = foobar(s, x);

	free(s);

	return enif_make_int(env, result);
}

static ErlNifFunc nif_funcs[] = {
    {"hello", 0, hello},
	{"foobar", 2, foobar_nif},
};

ERL_NIF_INIT(libexperiment, nif_funcs, NULL, NULL, NULL, NULL)
