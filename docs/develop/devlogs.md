[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Development Devlogs

# Development Devlogs

This acts as both official documentation and devlogs.

## Contents

- [Devlogs](#devlogs)

## Devlogs

<details>
<summary><strong>Nov. 21, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 09:59 PM +08:00 | Documented SVGO v4 for stripping editor metadata and optimizing custom SVG icons while preserving their scaling coordinates. |
| 01:58 PM +08:00 | Added hover handling to `LiquidButton`. The button stores hover state under its element ID and darkens its surface on pointer entry. Callers can also register a hover callback. |
| 01:28 PM +08:00 | Added a render-time assertion that a `LiquidButton` has a label, an icon, or both. |
| 12:33 PM +08:00 | Made `LiquidSurface` accept GPUI children. The children determine the surface's content layout and render above its background canvas. |
| 11:16 AM +08:00 | Made the liquid surface radius transition from square corners at zero through superellipse corners to semicircular ends at half the shorter side. The curve exponent decreases from 4 to 2 as the radius increases. |
| 10:34 AM +08:00 | Restored the liquid surface's 12 px corner radius. The fill and borders now trace quarter-superellipses joined by straight edges. The radius is capped at half the shorter side. Extracted the curve samples into a corner helper so the outline code handles the straight edges and connects the corners. |
| 10:32 AM +08:00 | Replaced the liquid surface's Bézier corners with a shared fourth-power superellipse path for the fill and borders. The canvas bounds determine the horizontal and vertical radii. |

</details>

<details>
<summary><strong>Nov. 19, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 11:52 AM +08:00 | Set a repository-wide rule to capture development discussions in dated devlogs. Added a documentation guide for breadcrumbs, tables of contents, devlogs, and collapsible sections. Checked the root `AGENTS.md` against the published guidance and confirmed that it uses the expected location and standard Markdown. |

</details>
