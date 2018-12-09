# swayfocus
Daemon for reducing opacity of non-focused [sway](https://github.com/swaywm/sway/) windows.

## Installation
```
~ $ cargo install swayfocus
```

## Usage
```
USAGE:
    swayfocus [FLAGS] <opacity>

FLAGS:
    -d, --daemonize    Run in the background.
    -h, --help         Prints help information
    -V, --version      Prints version information

ARGS:
    <opacity>    Opacity to be used for non focused windows (in interval 0..1)

```
