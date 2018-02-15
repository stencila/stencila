/*
  A graph of dependencies between Stencila repositories. To generate "repositories.svg" from this file:
*/
digraph G { 
  graph [fontname = "lato"] 
  node [fontname = "lato"] 
  edge [fontname = "lato"] 
 
  /* 
    Nodes for repositories, colour coded by main language 
    using Github language colours from https://github.com/ozh/github-colors/blob/master/colors.json 
  */ 
   
  node [shape=box, style=filled] 
  define(`lang_cpp',`color="#f34b7d"') 
  define(`lang_css',`color="#563d7c"') 
  define(`lang_js',`color="#f1e05a"') 
  define(`lang_nix',`color="#7e7eff"') 
  define(`lang_py',`color="#3572A5"') 
  define(`lang_r',`color="#198CE7"') 
   
  stencila [label="stencila", lang_js] 
 
  mini [lang_js] 
  convert [lang_js] 
 
  subgraph cluster_hosts {
    label="Hosts"
    pencolor=lightgray

    nodejs [lang_js]
    r [lang_r]
    py [lang_py]
    cloud [lang_js]
  }
 
  subgraph cluster_environments {
    label="Environments"
    pencolor=lightgray

    libcore
    images [lang_nix] 
  }
   
  subgraph cluster_deployments {
    label="Deployments"
    pencolor=lightgray
    rank=sink

    cli [lang_js]
    desktop [lang_js]
    hub [lang_py]
  }
 
  /* 
    Edges for dependencies between nodes 
  */ 
   
  edge [color=black] 
  mini -> stencila 
  
  stencila -> nodejs 
  convert -> nodejs

  nodejs -> cli 
  nodejs -> desktop 
  
  nodejs -> images 
  r -> images 
  py -> images 
 
  images -> cloud 
 
  subgraph cluster_key { 
    label="Key" 
    rank=sink 
 
    Nix [lang_nix] 
    Python [lang_py] 
    R [lang_r] 
  } 
} 
