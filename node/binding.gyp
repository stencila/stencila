{
  "targets": [
    {
      "target_name": "extension",
      "sources": [ "extension.cpp" ],
      "cflags_cc": ["-Wall", "-std=c++11", "-fexceptions"],
      "cflags_cc!": ["-fno-exceptions"],
      "include_dirs": [
        "<!(node -e \"require('nan')\")",
        ".",
        "./build",
        "../cpp",
        "../cpp/requires/boost"
      ],
      "link_settings": {
        "libraries": [
          "-lstencila"
        ],
        "ldflags": [
          "-L../cpp/library/"
        ]
      }
    },
    {
      "target_name": "action_after_build",
      "type": "none",
      "dependencies": [ "<(module_name)" ],
      "copies": [
        {
          "files": [ "<(PRODUCT_DIR)/<(module_name).node" ],
          "destination": "<(module_path)"
        }
      ]
    }
  ]
}
