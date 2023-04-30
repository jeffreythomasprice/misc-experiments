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
        "src/cpp/addon.cpp",
        "src/cpp/fuse-utils.cpp",
        "src/cpp/logging.cpp",
        "src/cpp/thread-utils.cpp",
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