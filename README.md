# Rust fswatcher

Watches a file, directory, or glob, and runs a command when it changes.  There's
probably already a million such things in existence, but writing code is more
fun than googling.

## Installation

* Clone the repository
* `cargo build --release`
* `cargo install --path .`

## Usage:

* Simple:

	`fswatch . ruby reload_my_app.rb`

* With exclude pattern (regex!  not glob!):

	`fswatch . ruby reload_my_app.rb --exclude='dist/.*'`

* With custom event debounce timing:

	`fswatch . ruby reload_my_app.rb --debounce-interval 10`

Of course run `fswatch --help` to get more detailed information.
