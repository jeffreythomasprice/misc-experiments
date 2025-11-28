#include <erl_nif.h>

#include <SDL3/SDL.h>
#include <SDL3/SDL_revision.h>

static ERL_NIF_TERM sdl_version_compiled(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_int(env, SDL_VERSION);
}

static ERL_NIF_TERM sdl_version_linked(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_int(env, SDL_GetVersion());
}

static ERL_NIF_TERM sdl_version_major(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    int x;
    if (!enif_get_int(env, argv[0], &x)) {
        return enif_make_badarg(env);
    }
    return enif_make_int(env, SDL_VERSIONNUM_MAJOR(x));
}

static ERL_NIF_TERM sdl_version_minor(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    int x;
    if (!enif_get_int(env, argv[0], &x)) {
        return enif_make_badarg(env);
    }
    return enif_make_int(env, SDL_VERSIONNUM_MINOR(x));
}

static ERL_NIF_TERM sdl_version_micro(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    int x;
    if (!enif_get_int(env, argv[0], &x)) {
        return enif_make_badarg(env);
    }
    return enif_make_int(env, SDL_VERSIONNUM_MICRO(x));
}

static ERL_NIF_TERM sdl_get_revision_compiled(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_string(env, SDL_REVISION, ERL_NIF_LATIN1);
}

static ERL_NIF_TERM sdl_get_revision_linked(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_string(env, SDL_GetRevision(), ERL_NIF_LATIN1);
}

static ERL_NIF_TERM sdl_get_error(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_string(env, SDL_GetError(), ERL_NIF_LATIN1);
}

static ERL_NIF_TERM sdl_clear_error(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    SDL_ClearError();
    return enif_make_int(env, 0);
}

static ERL_NIF_TERM sdl_init(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    int flags;
    if (!enif_get_int(env, argv[0], &flags)) {
        return enif_make_badarg(env);
    }
    return enif_make_int(env, SDL_Init(flags));
}

static ERL_NIF_TERM sdl_init_video(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    return enif_make_int(env, SDL_INIT_VIDEO);
}

/*
TODO rest of the init flags
SDL_INIT_AUDIO
SDL_INIT_VIDEO
SDL_INIT_JOYSTICK
SDL_INIT_HAPTIC
SDL_INIT_GAMEPAD
SDL_INIT_EVENTS
SDL_INIT_SENSOR
SDL_INIT_CAMERA
*/

static ERL_NIF_TERM sdl_quit(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    SDL_Quit();
    return enif_make_int(env, 0);
}

static ErlNifFunc nif_funcs[] = {
    {"sdl_version_compiled", 0, sdl_version_compiled},
    {"sdl_version_linked", 0, sdl_version_linked},
    {"sdl_version_major", 1, sdl_version_major},
    {"sdl_version_minor", 1, sdl_version_minor},
    {"sdl_version_micro", 1, sdl_version_micro},
    {"sdl_get_revision_compiled", 0, sdl_get_revision_compiled},
    {"sdl_get_revision_linked", 0, sdl_get_revision_linked},
    {"sdl_get_error", 0, sdl_get_error},
    {"sdl_clear_error", 0, sdl_clear_error},
    {"sdl_init", 1, sdl_init},
    {"sdl_init_video", 0, sdl_init_video},
    {"sdl_quit", 0, sdl_quit},
};

ERL_NIF_INIT(libsdl, nif_funcs, NULL, NULL, NULL, NULL)
