{
  "targets": [
    {
      "target_name": "extension",
      "sources": ["extension.cpp"],
      "cflags_cc": ["-Wall", "-std=c++11", "-fexceptions"],
      "cflags_cc!": ["-fno-exceptions"],
      "xcode_settings": {
        "MACOSX_DEPLOYMENT_TARGET": "10.7",
        'CLANG_CXX_LANGUAGE_STANDARD': 'c++11',
        'CLANG_CXX_LIBRARY': 'libc++',
        'GCC_ENABLE_CPP_EXCEPTIONS': 'YES',
        'GCC_ENABLE_CPP_RTTI': 'YES',
        "OTHER_CPLUSPLUSFLAGS": [
          "-Wall", "-std=c++11", "-fexceptions"
        ],
      },
      "include_dirs": [
        "<!(node -e \"require('nan')\")",
        ".",
        "../cpp",
        "../cpp/build/requires/boost"
      ],
      "link_settings": {
        "libraries": [
          "-lstencila"
        ],
        "ldflags": [
          "-L../../cpp/build/library"
        ]
      }
    },
    {
      "target_name": "action_after_build",
      "type": "none",
      "dependencies": ["<(module_name)"],
      "copies": [
        {
          "files": ["<(PRODUCT_DIR)/<(module_name).node"],
          "destination": "<(module_path)"
        }
      ]
    }
  ]
}
