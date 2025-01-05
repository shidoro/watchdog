# Watchdog

*A personal project I started for learning Rust.*  
Watchdog is a lightweight, language-agnostic file watcher that can run or build commands when your code changes. It uses [notify](https://github.com/notify-rs/notify) under the hood to monitor file events.


Note: at the moment Watchdog cannot watch itself.

---

## Configuration

Place a **`watchdog.toml`** in your project root. Example with both `[run]` and `[build]`:

```toml
[extend]
extendables = [
  { extendable_type = "git", path = ".gitignore" },
]

[exclude]
files = [
  { path = ".git" },
]

[run]
command = "cargo"
args = [ "run" ]
precompile = true
origin = "./nested/path"

[build]
command = "cargo"
args = [ "build" ]
origin = "./nested/path"
```
You can omit `[build]` if you don’t need a separate build step:
```toml
[extend]
extendables = [
  { extendable_type = "git", path = ".gitignore" },
]

[exclude]
files = [
  { path = ".git" },
]

[run]
command = "node"
args = [ "src/index.js" ]
```
* `[extend]`: Additional ignore patterns.
* `[exclude]`: Directories/files to skip.
* `[run]`: The command to run on each file change (can be any executable).
  * `precompile`: if true it requires `[build]`.
  * `origin`: if you have a nested project structure you can specify a directory relative to the root project, where to run the command from.
* `[build]`: Optional; if present, Watchdog can do a separate build step before running.

## Installation

Since this crate isn’t published yet, build from source:
```bash
git clone https://github.com/yourusername/watchdog.git
cd watchdog
cargo build --release
```
Then `cd` into your project and run:
```bash
/<your-path>/target/release/watchdog
```

## License
MIT License

Copyright (c) 2024 Donald Roshi

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
