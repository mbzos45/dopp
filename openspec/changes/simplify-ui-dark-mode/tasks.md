## 1. UI Simplification

- [x] 1.1 Remove header, subtitle, and metadata sections from container list layout
- [x] 1.2 Render each container row with name and action buttons only
- [x] 1.3 Keep refresh control as the only global UI control

## 2. Dark Mode

- [x] 2.1 Define light/dark color tokens in `src/app.css`
- [x] 2.2 Apply `prefers-color-scheme` to switch themes automatically
- [ ] 2.3 Verify contrast and button styles in both themes

## 3. Window Size Behavior

- [x] 3.1 Add container count tracking for initial and max observed counts
- [x] 3.2 Compute window size from container count with row height and padding constants
- [x] 3.3 Resize window on initial load and on refresh when count increases
- [x] 3.4 Skip resize when count decreases

## 4. Validation

- [ ] 4.1 Manually verify minimal UI layout against specs
- [ ] 4.2 Manually verify dark mode toggles with OS setting
- [ ] 4.3 Manually verify window expand-only behavior on refresh
