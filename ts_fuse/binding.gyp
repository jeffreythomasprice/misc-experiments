{
  "targets": [
    {
      "target_name": "addon",
      "cflags_cc": [
        "-std=c++20",
        "-fexceptions",
        "<!@(pkg-config fuse --cflags)"
      ],
      "link_settings": {
        "libraries": [
          "<!@(pkg-config fuse --libs)"
        ]
      },
      "sources": [
        "src/addon.cpp",
        "src/fuse-utils.cpp",
        "src/logging.cpp",
        "src/thread-utils.cpp",
      ],
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")"
      ],
      "defines": [
        "NAPI_CPP_EXCEPTIONS"
      ]
    }
  ]
}