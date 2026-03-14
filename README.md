# recon

A TUI dashboard for monitoring [Claude Code](https://claude.ai/claude-code) sessions running inside **tmux**.

See all your Claude sessions at a glance — what they're working on, which need your attention, and how much context they've consumed.

## Dashboard

```
┌─ recon — Claude Code Sessions ───────────────────────────────────────────────────────────────┐
│  #  Session          Git(Project::Branch)     Directory            Status  Model        …    │
│  1  api-refactor     myapp::feat/auth         ~/repos/myapp        ● Input Opus 4.6     …    │
│  2  debug-pipeline   infra::main              ~/repos/infra        ● Working Sonnet 4.6 …    │
│  3  write-tests      myapp::feat/auth         ~/repos/myapp        ● Working Haiku 4.5  …    │
│  4  code-review      webapp::pr-452           ~/repos/webapp       ● Idle  Sonnet 4.6   …    │
│  5  scratch           recon::main              ~/repos/recon        ● Idle  Opus 4.6     …    │
│  6  new-session      dotfiles::main           ~/repos/dotfiles     ● New   —            …    │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
j/k navigate  Enter switch  r refresh  q quit
```

- **Input** rows are highlighted — these sessions are blocked waiting for your approval
- **Working** sessions are actively streaming or running tools
- **Idle** sessions are done and waiting for your next prompt
- **New** sessions haven't had any interaction yet

## How it works

recon is built around **tmux**. Each Claude Code instance runs in its own tmux session.

```
┌─────────────────────────────────────────────────┐
│                    tmux server                   │
│                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
│  │ session:     │  │ session:     │  │ session: │ │
│  │ api-refactor │  │ debug-pipe   │  │ scratch  │ │
│  │              │  │              │  │          │ │
│  │  ┌────────┐  │  │  ┌────────┐  │  │ ┌──────┐ │ │
│  │  │ claude │  │  │  │ claude │  │  │ │claude│ │ │
│  │  └────────┘  │  │  └────────┘  │  │ └──────┘ │ │
│  └──────┬───────┘  └──────┬───────┘  └────┬─────┘ │
│         │                 │               │        │
└─────────┼─────────────────┼───────────────┼────────┘
          │                 │               │
          ▼                 ▼               ▼
    ┌──────────────────────────────────────────┐
    │               recon (TUI)                │
    │                                          │
    │  reads:                                  │
    │   • tmux list-panes → PID, session name  │
    │   • ~/.claude/sessions/{PID}.json        │
    │   • ~/.claude/projects/…/*.jsonl          │
    │   • tmux capture-pane → status bar text  │
    └──────────────────────────────────────────┘
```

**Status detection** inspects the Claude Code TUI status bar at the bottom of each tmux pane:

| Status bar text | State |
|---|---|
| `esc to interrupt` | **Working** — streaming response or running a tool |
| `Esc to cancel` | **Input** — permission prompt, waiting for you |
| anything else | **Idle** — waiting for your next prompt |
| *(0 tokens)* | **New** — no interaction yet |

**Session matching** uses `~/.claude/sessions/{PID}.json` files that Claude Code writes, linking each process to its session ID. No `ps` parsing or CWD-based heuristics.

## Install

```bash
cargo install --path .
```

Requires tmux and [Claude Code](https://claude.ai/claude-code).

## Usage

```bash
recon              # TUI dashboard
recon --json       # JSON output (for scripting)
recon launch       # Create a new claude session in the current directory
recon new          # Interactive new session form
```

### Keybindings

| Key | Action |
|---|---|
| `j` / `k` | Navigate sessions |
| `Enter` | Switch to selected tmux session |
| `r` | Force refresh |
| `q` / `Esc` | Quit |

## tmux config

The included `tmux.conf` provides keybindings to open recon as a popup overlay:

```bash
# Add to your ~/.tmux.conf
bind r display-popup -E -w 80% -h 60% "recon"        # prefix + r → dashboard
bind n display-popup -E -w 80% -h 60% "recon new"    # prefix + n → new session
bind X confirm-before -p "Kill session #S? (y/n)" kill-session
```

This lets you pop open the dashboard from any tmux session, pick a session with `Enter`, and jump straight to it.

## Features

- **Live status** — polls every 2s, incremental JSONL parsing
- **Git-aware** — shows repo name and branch per session
- **Token tracking** — input/output tokens with context window ratio
- **Model display** — shows which Claude model and effort level
- **Multi-session** — handles multiple sessions in the same repo without conflicts
- **JSON mode** — `recon --json` for scripting and automation
