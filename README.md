# xim
xim is an experimental text editor in a 'vim' like flavour using the xi-editor backend.
This project is based heavily on [xi-term](https://github.com/xi-frontend/xi-term)

## Goals
 * Keep it simple
 * Modal vim like behaviour
    * command mode
        * ~~u64 goto line~~
        * ~~'w' write~~
        * ~~'q' quit~~
        * ~~'wq' write and quit~~
        * 'd' delete line
        * 'v' visual mode
        * 'V' visual line mode
        * 'y' yank
        * 'p' put
    * search/replace mode
        * search
        * replace
    * status bar
        * ~~file name~~
        * ~~cursor pos~~
    * ~~simple error message~~
 * Syntax highlighting
 * LSP at some point

