[Pitaka](../../README.md) <span style="color: #888;">/</span> Development

# Development

This acts as both official documentation and devlogs.

## Contents

- [Documents](#documents)
- [Devlogs](#devlogs)

## Documents

| Document | Description |
| --- | --- |
| [Code Organization](code_organization.md) | Module responsibilities, dependency direction, and where new logic belongs. |
| [File Layout](file_layout.md) | Binary structures, offsets, and record payloads. |
| [Querying](querying.md) | Collection queries, relationships, and indexing. |
| [Findings](findings.md) | Shared record of verified issues and their resolutions. |
| [Documentation Rules](../rules.md) | Breadcrumbs, tables of contents, devlogs, and collapsible sections. |

## Devlogs

<details>
<summary><strong>Nov. 21, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
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
