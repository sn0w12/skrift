# Skrift

Skrift is a minimal, themeable text editor built with Rust and FLTK.

## Features

-   Customizable themes (colors, fonts, cursor style)
-   Configurable keyboard shortcuts (save, quit, reload, move lines)
-   Live config reload (changes to config file are applied instantly)
-   Status bar showing cursor position
-   Simple file open/save logic

## Building and Setup

1. **Install Rust**
   If you don't have Rust, install it from [rustup.rs](https://rustup.rs).

2. **Clone the repository**

    ```bash
    git clone https://github.com/yourusername/skrift.git
    cd skrift
    ```

3. **Install the binary**

    ```bash
    cargo install --path .
    ```

4. **Create a link to the binary**

    ```bash
    sudo ln -sf $HOME/.cargo/bin/skrift /usr/local/bin/skrift
    ```

5. **Run Skrift**
   Now you can run Skrift from anywhere:
    ```bash
    skrift [file]
    ```
    If no file is specified, `out.txt` will be used by default.

## Configuration

The config file is located at:

```
~/.config/skrift/config.skrift
```

You can customize themes and key bindings using TOML syntax.

### Example Config

```toml
[theme]
background = "#05101a"
foreground = "#e6f1ff"
selection_color = "#74c4c9"
negative_color = "#f72650"
font_family = "Courier"
font_size = 16
cursor_style = "simple"
cursor_flash = true
cursor_flash_interval = 0.5
scrollbar_style = "rounded"

[editor]
scroll_multiplier = 3

[bindings]
save = "Ctrl+S"
quit = "Ctrl+Q"
reload = "Ctrl+R"
move_line_up = "Alt+Up"
move_line_down = "Alt+Down"
open_file = "Ctrl+O"
```
