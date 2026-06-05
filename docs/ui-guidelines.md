# Unfour UI Guidelines

## Product style
Unfour is a compact desktop developer tool.
The UI should feel closer to VS Code, Cursor, TablePlus and Linear.
Avoid SaaS dashboard styling.

## Layout
- Use a fixed app shell.
- Left sidebar is for navigation.
- Main content uses tabs.
- Bottom panel is for logs and output.
- Module-specific details use a right-side inspector or drawer.
- Avoid nested card layouts.

## Density
- Use compact spacing.
- Toolbar height: 36px.
- Tab height: 34px.
- Table row height: 30px.
- Default font size: 13px.
- Border radius: 4px to 6px.

## Components
- Reuse components from packages/ui.
- Do not create page-local button or input styles.
- Use Lucide icons only.
- Use shared color tokens only.
- Prefer icon buttons with tooltips for secondary actions.

## Visual hierarchy
- One primary action per panel.
- Secondary actions go into context menus or dropdown menus.
- Avoid oversized headings.
- Avoid large empty areas unless showing an empty state.

## Interaction
- All common actions should support keyboard shortcuts.
- Context menus should be available for tree nodes and table rows.
- Loading, empty, error and disconnected states must be implemented.

## Forbidden patterns
- No large dashboard cards.
- No random gradients.
- No excessive rounded corners.
- No mixed icon libraries.
- No module-specific design language.