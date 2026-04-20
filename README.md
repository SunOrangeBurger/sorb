# sorb

A custom, robust UNIX-style Rust shell containing a hidden, zero-external-dependency terminal game engine.

## Overview
**`sorb`** operates as a functional terminal REPL with intelligent tab completion and persistent command history. It utilizes `rustyline` for fluent syntax control, line-arrow history tracking, and tab completion support. The shell uses `shell-words` for accurate UNIX argument/quote parsing, and hooks globally into `ctrlc` to perfectly emulate the SIGINT behavior of shells such as `bash` and `zsh` — blocking `Ctrl+C` from terminating the shell itself, but accurately forwarding it to active blocking child processes.

Tab completion works across built-in commands, system executables in PATH, and file paths. Command history is automatically persisted to `~/.sorb_history` and survives across sessions.

## Installation & Usage

### Quick Install(not functional at the momement, still working on it!!)
Run the installation script to build and install sorb to `~/.local/bin`:

```bash
./install.sh
```

Ensure `~/.local/bin` is in your PATH by adding this to your `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Manual Build
1. Ensure you have Rust and Cargo installed.
2. Clone this repository.
3. Build and run:
   ```bash
   cargo run --release
   ```
4. Once spawned, `sorb` will interpret standard CLI utility commands exactly like any other shell (e.g., `ls -l`, `pwd`, `git status`). 
5. Supported built-ins:
   - `cd <path>` (supports `~` for home directory)
   - `exit`

### Tab Completion
Press `Tab` while typing to trigger intelligent completion:
*   Built-in commands (`cd`, `exit`)
*   System executables found in PATH
*   File and directory paths for command arguments
*   Easter egg game commands

### Command History
Navigate command history using up/down arrow keys. History is automatically saved to `~/.sorb_history` and persists across sessions.

### Setting `sorb` as your default shell (Linux / macOS)
If you want to use `sorb` as your actual login shell:

```bash
# 1. Build the release binary
cargo build --release

# 2. Copy the binary to a universal system path (requires sudo)
sudo cp target/release/sorb /usr/local/bin/

# 3. Add the path to the system's list of allowed shells
echo "/usr/local/bin/sorb" | sudo tee -a /etc/shells

# 4. Change your user's default shell
chsh -s /usr/local/bin/sorb
```

*(Note: Log out and log back in, or open a new terminal window for the changes to take effect.)*

### Uninstall
Remove sorb from your system:

```bash
./uninstall.sh
```


## Secret Games (Easter Eggs)

`sorb` contains a custom-built, raw-mode terminal game engine. If the shell detects a specific hidden command string, it pauses external execution, drops the terminal into raw alternative-screen mode, and launches a real-time fixed-step game loop running at ~60 ticks per second natively in the UI.

High scores are permanently persisted via JSON locally to `~/.sorb/scores.json`.

Type the following commands directly into the `sorb $` prompt:
*   `sorb-snake` : A classic Snake clone (`WASD` / Arrows, `Space` on Game Over).
*   `sorb-dino` : An infinite side-scroller jumping game (`Space`).
*   `sorb-flappy` : Infinite flap physics runner (`Space`).
*   `sorb-tetris` : Full tetromino puzzle implementation (Arrows, `Space` for hard drop).
*   `sorb-invaders` : Space Invaders clone with AABB hitboxes and increasing wave tiers (`Left/Right`, `Space` to shoot).

Press `Q` or `Esc` during any game to instantly drop back completely unharmed into your standard `sorb` programming shell context.
