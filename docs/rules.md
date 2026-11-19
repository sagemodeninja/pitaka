[Pitaka](../README.md) <span style="color: #888;">/</span> Documentation Rules

# Documentation Rules

This acts as both official documentation and devlogs.

## Contents

- [Breadcrumbs](#breadcrumbs)
- [Table of contents](#table-of-contents)
- [Devlogs](#devlogs)
- [Collapsible sections](#collapsible-sections)

## Breadcrumbs

Put breadcrumbs on the first line, followed by a blank line and the page title. Link `Pitaka` to the root README and link each parent section to its index using paths relative to the page. Separate items with `<span style="color: #888;">/</span>` and leave the current page title as plain text.

```markdown
[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Querying
```

## Table of contents

Add a `## Contents` section near the top. Use a bulleted list of links to the page’s headings in page order. Nest links for subsections when they help readers navigate, and use the heading’s rendered anchor, such as `#table-of-contents`.

## Devlogs

Put devlogs under a `## Devlogs` heading. Use one collapsible section per date, with newest dates first. Put a Markdown table with `Time` and `Logs` columns inside each section and sort its rows by timestamp, newest first. Record the developer’s local time with its UTC offset. Keep entries brief and merge related follow-ups when one row can still capture the sequence. Give distinct decisions or milestones their own timestamp.

<details>
<summary><strong>Nov. 19, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 11:56 AM +08:00 | Established the formatting rules for documentation pages and clarified that timestamps within each date run newest first. |

</details>

## Collapsible sections

Wrap the fold in `<details>`. Give it a title in `<summary>` and put `<br>` on the next line. Leave a blank line after `<br>` and before `</details>` so Markdown inside the fold renders correctly.

```markdown
<details>
<summary><strong>Section title</strong></summary>
<br>

Section content goes here.

</details>
```
