{
  "targets": [
    {
      "target_name": "extension",
      "sources": [ "extension.cpp" ],
      "cflags_cc": ["-Wall", "-std=c++11", "-fexceptions"],
      "cflags_cc!": ["-fno-exceptions"],
      "include_dirs": [
        "<!(node -e \"require('nan')\")",
        ".", "./build",
        "../cpp",
        "../build/current/cpp/requires/boost"
      ],
      "link_settings": {
        "libraries": [
          "-lstencila"
        ],
        "ldflags": [
          "-L../../build/current/cpp/library/"
        ]
    }
    }
  ]
}
