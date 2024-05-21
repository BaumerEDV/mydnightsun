# Feature Roadmap and Tracking

## Completed
- open file from provided path
- scroll vertically inside file
- scroll horizontally inside file
- toggle line wrapping
- filter which lines are displayed (and in which color) with user-defined regex read from a provided JSON-file

## Vision / Roadmap
- show keybind help info
- toggle filters live in the viewer
- simple live text filter
- edit filters in the application and persist them
- shortcut customization
- support stdio rather than just a file flag
- support live filtering for files "trickling in" through stdio
- performance increase through multithreading
- handle files that do not fit into RAM
- can't scroll further to the right than longest line (on screen? account for line wrapping toggle?)
- enable scrolling for short files with lines that when wrapped will be larger than the screen


## Roadstops / Immediate Goals
- filter json documentation
- jump to start of line horizontally

## Technical Tasks
- graceful error handling
- break up into modules
- CI tests
- github action builds
- nix derivation
- move command line arguments to clap
- figure out if the ratatui demo way is actually the correct way to render - seems like a lot of resources. Possibly use callback on change instead?
- graciously exit and tear down the terminal modifications if there is a panic
