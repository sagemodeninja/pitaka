[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Code Organization

# Code Organization

Keep code near the feature that owns it. Choose modules by responsibility and
dependencies, not whether code is logic, shared, or generic.

| Location | Responsibility | Examples |
| --- | --- | --- |
| `src/core` | Pitaka data, rules, persistence, and state. | Documents, collections, transactions, money, settings. |
| `src/utils` | General helpers independent of Pitaka concepts. | Binary reads, string helpers. |
| `src/views` | Screens and user interaction. | Home screen. |
| `src/components` | Reusable UI elements. | Launch action component. |
| Application integration | Connect user actions to core. | Menus, key bindings, dialogs, action handlers. |

A collection query builder belongs in `src/core/document/collection/query.rs`
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
