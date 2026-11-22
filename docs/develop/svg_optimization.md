[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> SVG Icon Optimization

# SVG Icon Optimization

This acts as both official documentation and devlogs.

## Contents

- [Optimizing SVG icons](#optimizing-svg-icons)

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
