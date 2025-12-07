#include <erl_nif.h>

#include <SDL3/SDL.h>
#include <SDL3/SDL_revision.h>

char* get_c_string_arg(ErlNifEnv* env, ERL_NIF_TERM arg, unsigned int *len) {
    unsigned int len2;
    enif_get_string_length(env, arg, &len2, ERL_NIF_LATIN1);
    printf("TODO getting str length, len = %d\n", len2);
    if (len) {
        *len = len2;
    }
    char *result = malloc(len2 + 1);
    if (!enif_get_string(env, arg, result, len2 + 1, ERL_NIF_LATIN1)) {
        free(result);
        return NULL;
    }
    return result;
}

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
SDL_INIT_VIDEO all
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

static ERL_NIF_TERM sdl_create_window_and_renderer(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {
    char *title = get_c_string_arg(env, argv[0], NULL);
    if (!title) {
        return enif_make_badarg(env);
    }
    printf("TODO title = %s\n", title);

    int width;
    if (!enif_get_int(env, argv[1], &width)) {
        free(title);
        return enif_make_badarg(env);
    }

    int height;
    if (!enif_get_int(env, argv[2], &height)) {
        free(title);
        return enif_make_badarg(env);
    }

    int flags;
    if (!enif_get_int(env, argv[3], &flags)) {
        free(title);
        return enif_make_badarg(env);
    }

    SDL_Window *window = NULL;
    SDL_Renderer *renderer = NULL;
    if (!SDL_CreateWindowAndRenderer(title, width, height, flags, &window, &renderer)) {
        return enif_make_tuple2(env, enif_make_atom(env, "Error"), enif_make_atom(env, "Nil"));
    }

    ERL_NIF_TERM window_term = enif_make_resource(env, window);
    enif_release_resource(window);

    ERL_NIF_TERM renderer_term = enif_make_resource(env, renderer);
    enif_release_resource(renderer);

    return enif_make_tuple2(env, window_term, renderer_term);
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
    {"sdl_create_window_and_renderer", 4, sdl_create_window_and_renderer},
};

ERL_NIF_INIT(libsdl, nif_funcs, NULL, NULL, NULL, NULL)
