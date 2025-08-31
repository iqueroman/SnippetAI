# Clipboard Screenshot Assistant — Project Specification

## IMPORTANT: DO NOT DO ANYTHING THAT WAS NOT EXPLICTLY ASKED

## 1) Summary
A lightweight desktop app that **watches the clipboard for screenshots** (e.g., `Win + Shift + S` on Windows), sends the image **with a user-defined system prompt** to a selected **LLM provider** (via **OpenRouter**), and **renders the answer fast**—**without stealing focus** from the currently active Chrome tab/window.

---

## 2) Non-Negotiable Constraints
- **Do not steal focus:** After the user snips, **Chrome stays the active window**.  
  - The app must **not** call `focus()`, `activate()`, or equivalent on clipboard events.
  - **Fast response path:** Keep latency low (target ≤ 2–3 s end-to-end with cached model/provider setup).

---

## 3) UI (Simple, Modern)
- **Layout:** Minimal, responsive window (when opened by the user).
- **Fields:**
  - **System Prompt** (multiline text area).
  - **Model Provider** (dropdown: OpenRouter)
  - **API Key** input (masked). Support **OpenRouter** to simplify.
  - **Model** (dropdown populated from provider; default settable).

---

## 4) Core Workflow
1. User presses `Win + Shift + S` (Windows Snipping Tool).
2. OS places the snipped image in the **clipboard**.
3. App’s clipboard watcher detects **new image blob**.
4. App packages: `{ systemPrompt, screenshotImage, selectedModel }`.
5. Send request to **provider client** (OpenRouter).
6. Receive **LLM response**; render to output pane; **do not** bring window to foreground.

---

## 5) Functional Requirements
- **Clipboard Monitoring**
  - Detect **image** type specifically.
  - Debounce rapid events (e.g., 250–500 ms).
  - Optional: ignore non-image clipboard updates.
- **Provider Abstraction**
  - Switch providers without changing other code (Strategy pattern).
- **Prompting**
  - Combine a **system prompt** with an **image** input (multimodal).
- **Output**
  - Show text response.
- **Persistence**
  - Persist UI settings (prompt, provider, model).
  - Store **API keys** securely (OS Keychain where possible).
- **Error Handling**
  - Network/API errors → non-blocking toast with retry.
  - Invalid key → inline validation + guidance.

---

## 6) Acceptance Criteria (Objective)
- **Focus:** After snipping, the **Chrome tab remains active**. The app **never** steals focus on clipboard events.
- **Latency:** With a small screenshot, default settings, OpenRouter: **≤ 3 seconds** median response under normal network.
- **Robustness:** Two consecutive snips within 1 second → the app processes **the most recent image** (debounced).
- **Security:** API keys saved via **OS secure storage** (Windows Credential Manager / macOS Keychain). No plaintext on disk.
- **Stability:** The watcher runs ≥ 8 hours without memory leaks or crashes.

---

## 7) Suggested Architecture

- **Tauri** → low footprint, fast.
- **UI:** Svelte front-end, minimal components.

## IMPORTANT: DO NOT DO ANYTHING THAT WAS NOT EXPLICTLY ASKED