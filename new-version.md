# Focus-Write: Future Roadmap

This document outlines planned enhancements for the Focus-Write minimal editor. Features are categorized by their impact on the user experience.

## 1. Typewriter Feel
- [x] **Mechanical Sounds**: Subtle audio feedback for key presses and a "ding" sound on carriage returns (Enter key).
- [x] **Smooth Typewriter Scrolling**: Refine the viewport logic to ensure the active line transitions smoothly as the cursor moves.

## 2. Writing Goals & Zen
- [ ] **Daily/Session Goals**: Status bar indicator for word count targets (e.g., "342 / 500 words").
- [ ] **Pure Zen Mode**: Shortcut to toggle all UI elements (status bar, word counts) for a distraction-free experience.
- [ ] **Read-Only Mode**: A safety toggle to prevent accidental text changes during review sessions.

## 3. Visual Refinements
- [x] **Color Themes**:
    - *Sepia*: Warm paper background with dark brown text.
    - *E-Ink*: High-contrast black and white.
    - *Night*: Deep blue/black backgrounds.
- [x] **Dynamic Typography**: Keyboard shortcuts (`Ctrl` + `+`/`-`) to adjust font size and line spacing on the fly.
- [ ] **Subtle Markdown Support**: Light colorization for headers (`#`), emphasis (`*`), and blockquotes (`>`).

## 4. Technical Reliability
- [x] **Auto-Save**: Background saving to a `.backup` file every 60 seconds.
- [x] **Undo/Redo History**: Implementation of a basic history stack for `Ctrl+Z` and `Ctrl+Y` functionality.
- [x] **Reading Time**: An estimate in the status bar (e.g., "3 min read").

## 5. File & Export
- [x] **PDF Export**: Generate clean, formatted PDF documents from the raw text.
- [x] **Session Summary**: A statistics pop-up upon exit showing total words written and time elapsed.

gemini --resume d98ca3ca-bffc-4de3-9692-864dac4b1674
