# Feature Roadmap and Tracking

## Completed
- open file from provided path
- scroll vertically inside file

## Vision / Roadmap
- display navigable view of text file from disk 
- filter which lines are displayed with user-defined regex read from a provided file
- color lines matching user-defined regex and color instructions
- show keybind help info
- toggle filters live in the viewer
- simple live text filter
- edit filters in the application and persist them
- shortcut customization
- support stdio rather than just a file flag
- support live filtering for files "trickling in" through stdio
- performance increase through multithreading
- handle files that do not fit into RAM


## Roadstops / Immediate Goals
- make that file horizontally navigable in the TUI as a single screen application
- load filters from a file
- apply filters
- add coloring to filters and display those colors

## Technical Tasks
- CI tests
- github action builds
- nix derivation
- move command line arguments to clap
- figure out if the ratatui demo way is actually the correct way to render - seems like a lot of resources. Possibly use callback on change instead?
- graciously exit and tear down the terminal modifications if there is a panic
