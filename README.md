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
* [x] 'Esc' returns to xim mode
* [ ] syntax highlighting
* [ ] xim mode
    * [x] 'arrows' move up/down/left/right
    * [x] 'PageUp|Down' move page up/down
    * [x] 'Home|End' move to line start/end
    * [x] 'j|k|l' move down/up/right
    * [x] 'i' insert mode
    * [x] 'v' visual mode
    * [ ] 'V' visual line mode
    * [x] ':' command mode
    * [x] '/' search mode
    * [x] 'y' copy
    * [x] 'p' paste
    * [x] 'd' cut
    * [x] 'u' undo
    * [x] 'r' redo
    * [ ] 'dd' delete line
* [ ] visual mode
    * [x] 'arrows' move selection up/down/left/right
    * [x] 'PageUp|Down' move selection page up/down
    * [x] 'Home|End' move selection to line start/end
    * [x] 'j|k|l' move selection down/up/right
    * [x] 'y' copy
    * [x] 'p' paste
    * [x] 'd' cut
    * [x] 'i' insert mode
* [ ] command mode
    * [x] u64 goto line
    * [x] 'w' write
    * [x] 'q' quit
    * [x] 'wq' write and quit
* [ ] search mode
    * [ ] highlight
    * [ ] selection
    * [ ] search
        * [ ] next
        * [ ] prev
    * [ ] replace
        * [ ] all
* [ ] error mode
    * [x] error message
    * [ ] debug dialog message
* [ ] plugin mode
    * [ ] notify
* [ ] status bar // FIXME: use xi backend for status bar
    * [x] file name
    * [x] cursor pos


