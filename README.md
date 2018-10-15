# xim
xim is a experimental modal text editor using the xi-editor backend inspired by
vim. Focusing on emulating vim's modal states (vim,insert,visual) and behaviour
as much as possible, by translating all corresponding actions to the xi core.
Not intended to be a vim rewrite, rather a familiar expirience.

This project is based on [xi-term](https://github.com/xi-frontend/xi-term), just as [xim](https://github.com/xi-frontend/xim).
Both use the same base code from xi-term and have, due to naming, similar
portions of code. A rename or merge to some point should be given consideration.

Since the xi-editor is to implement a vi mode, this project may become obsolete
depending on how it is implemented.

# Status
* xim
    * 'Esc' returns to xim mode
    * xim mode
        * 'i' insert mode
        * 'v' visual mode
        * ':' command mode
        * '/' search mode
        * 'y' copy
        * 'p' paste
        * 'd' cut
        * 'u' undo
        * 'r' redo
        * 'd' delete line
    * visual mode
        * 'y' copy
        * 'p' paste
        * 'd' cut
        * 'i' insert mode
    * command mode
        * u64 goto line
        * 'w' write
        * 'q' quit
        * 'wq' write and quit
    * status bar
        * file name
        * cursor pos
    *  error state with message

# TODO
* xim
    * xim mode
        * 'V' visual line mode
        * 'd' delete line
    * search/replace mode
        * search
        * replace
        * highlight
    * syntax highlighting
    * plugin handling


