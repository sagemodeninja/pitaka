[Pitaka](../../README.md) <span style="color: #888;">/</span> Development

# Development

This acts as both official documentation and devlogs.

## Contents

- [Documents](#documents)
- [Optimizing SVG icons](#optimizing-svg-icons)
- [Devlogs](#devlogs)

## Documents

| Document | Description |
| --- | --- |
| [Code Organization](code_organization.md) | Module responsibilities, dependency direction, and where new logic belongs. |
| [File Layout](file_layout.md) | Binary structures, offsets, and record payloads. |
| [Querying](querying.md) | Collection queries, relationships, and indexing. |
| [Findings](findings.md) | Shared record of verified issues and their resolutions. |
| [Documentation Rules](../rules.md) | Breadcrumbs, tables of contents, devlogs, and collapsible sections. |

## Optimizing SVG icons

Use SVGO v4 to strip editor metadata and optimize SVG markup before adding a custom icon to `assets/icons/`. With Node.js and npm installed, run this from the repository root:

```sh
npx svgo@4 assets/icons/trash.svg -o /tmp/trash.min.svg
```

Replace `trash.svg` with the icon you’re working on. The command writes a separate file so you can compare it with the original. SVGO v4 preserves an existing `viewBox` by default. Check that the optimized SVG has a `viewBox` and looks right at the size GPUI will display it. Then copy it over the original:

```sh
cp /tmp/trash.min.svg assets/icons/trash.svg
```

[SVGO command-line usage](https://github.com/svg/svgo#command-line-usage) · [SVGO v4 defaults](https://github.com/svg/svgo/blob/main/docs/06-migrations/01-migration-from-v3-to-v4.mdx)

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
