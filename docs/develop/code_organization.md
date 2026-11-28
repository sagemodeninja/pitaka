[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Code Organization

# Code Organization

This acts as both official documentation and devlogs.

## Contents

- [Dependencies](#dependencies)
- [Devlogs](#devlogs)

Keep code near the feature that owns it. Choose modules by responsibility and
dependencies, not whether code is logic, shared, or generic.

| Location | Responsibility | Examples |
| --- | --- | --- |
| `src/core` | Pitaka data, rules, persistence, and state. | Vaults, collections, transactions, money, settings. |
| `src/utils` | General helpers independent of Pitaka concepts. | Binary reads, string helpers. |
| `src/views` | Screens and user interaction. | Home screen. |
| `src/components` | Reusable UI elements. | Launch action component. |
| Application integration | Connect user actions to core. | Menus, key bindings, dialogs, action handlers. |

A collection query builder belongs in `src/core/vault/collection/query.rs`
because it operates on `Collection<T>`, even when generic over `T`.

## Dependencies

```text
UI / application integration -> core -> utils
UI / application integration -> utils
```

Core should not depend on UI code. General utilities should not depend on core.
Prefer standard library helpers and keep implementation details private.

> Existing exceptions: `utils/actions.rs`, `utils/menu.rs`, and `utils/keybinding.rs`
contain application integration; `utils/dir.rs` includes Pitaka configuration.
Revisit their placement when changing that area.

## Devlogs

<details>
<summary><strong>Nov. 28, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 10:44 PM +08:00 | Confirmed visually on macOS that `LiquidSurface` draws solid red through `Window::paint_backdrop` after removing the temporary button color marker. |
| 10:39 PM +08:00 | Wired Metal backdrop batches to a separate instance binding. The renderer packs backdrop geometry as quads and reuses the quad draw routine with the red test pipeline. |
| 10:28 PM +08:00 | The experimental GPUI backdrop paint method submits one primitive for the full region. Removed the border-only quad splitting logic and verified the macOS build with `cargo check --offline`. |

</details>

<details>
<summary><strong>Nov. 22, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 01:54 PM +08:00 | Renamed the persistence module to `core::vault` and the main type to `PitakaVault`. Updated the documentation references and verified the rename with an offline build check. |

</details>
