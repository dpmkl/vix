# xim
xim is an experimental text editor in a 'vim' like flavour using the xi-editor backend.
This project is based heavily on [xi-term](https://github.com/xi-frontend/xi-term)

## Goals
 * Keep it simple
 * Modal vim like behaviour
    * ~~'Esc' returns to xim mode~~
    * xim mode
        * ~~'i' insert mode~~
        * ~~'v' visual mode~~
        * ~~':' command mode~~
        * ~~'/' search mode~~
        * ~~'y' copy~~
        * ~~'p' paste~~
        * ~~'d' cut~~
        * ~~'u' undo~~
        * ~~'r' redo~~
        * 'd' delete line
    * visual mode
        * ~~'y' copy~~
        * ~~'p' paste~~
        * ~~'d' cut~~
        * ~~'i' insert mode~~
    * command mode
        * ~~u64 goto line~~
        * ~~'w' write~~
        * ~~'q' quit~~
        * ~~'wq' write and quit~~
    * search/replace mode
        * search
        * replace
    * status bar
        * ~~file name~~
        * ~~cursor pos~~
    * ~~simple error message~~
 * Syntax highlighting
 * LSP at some point

