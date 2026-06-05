# Unfour Workspace UI Guidelines

## 1. Product Positioning

Unfour Workspace is a compact desktop developer tool.

It combines:

* SSH terminal
* Database management
* API debugging
* Workspace management
* Logs and developer utilities

The interface should feel like a unified IDE rather than a collection of unrelated web pages.

Reference style:

* VS Code and Cursor for application structure
* Linear for visual restraint
* TablePlus and DataGrip for database density
* Bruno and Insomnia for API debugging clarity
* Termius and Tabby for terminal workflows

Avoid SaaS dashboard styling.

---

## 2. Core Design Principles

### 2.1 Compact desktop layout

Unfour is a desktop productivity application.

Prefer:

* compact spacing
* high information density
* clear visual hierarchy
* restrained use of decoration
* keyboard-friendly interactions
* context menus and secondary menus

Avoid:

* oversized headings
* large blank areas
* mobile-first layouts
* marketing-page layouts
* excessive explanatory text
* large rounded cards

### 2.2 Unified application shell

All modules must use the same shell:

```text
┌─────────────────────────────────────────────┐
│ Global toolbar                             │
├────────────┬────────────────────────────────┤
│ Sidebar    │ Main tab area                  │
│ Explorer   │                                │
│            │                                │
├────────────┴────────────────────────────────┤
│ Bottom panel: logs / output / status        │
└─────────────────────────────────────────────┘
```

The UI must include:

* fixed global toolbar
* left navigation sidebar
* central tab-based workspace
* optional right-side inspector
* optional bottom panel
* compact status bar

Do not create module-specific shells.

---

## 3. Layout Rules

### 3.1 Global toolbar

The global toolbar should contain only application-level actions:

* workspace switcher
* global search
* command palette
* layout toggle
* settings
* account or sync state

Module-specific actions must not appear here.

### 3.2 Sidebar

The sidebar is for navigation and tree structures.

Examples:

* saved connections
* SSH hosts
* database schemas
* API collections
* workspace files

Rules:

* use compact tree rows
* use context menus for secondary operations
* show only essential inline actions
* avoid placing full forms inside the sidebar
* use drawers or dialogs for configuration

### 3.3 Main workspace

The main content area uses tabs.

Examples:

* terminal sessions
* SQL editors
* database tables
* API requests
* log views
* settings pages

Rules:

* tabs must support close action
* tabs may show modified or connection state
* tabs should reuse the same tab component
* module pages should not recreate their own navigation bars

### 3.4 Right-side inspector

Use a right-side inspector for secondary details:

* connection properties
* table metadata
* request history
* environment variables
* object properties

Do not place low-frequency settings directly in the main workspace.

### 3.5 Bottom panel

Use the bottom panel for:

* logs
* task output
* request trace
* connection diagnostics
* error details

The panel should support collapse and resizing.

---

## 4. Density Rules

Use compact desktop dimensions.

Recommended baseline:

| Element          | Height |
| ---------------- | -----: |
| Global toolbar   |   38px |
| Tab bar          |   34px |
| Sidebar tree row |   28px |
| Input            |   32px |
| Button           |   30px |
| Compact button   |   28px |
| Table row        |   30px |
| Status bar       |   24px |
| Section toolbar  |   34px |

Recommended border radius:

| Element |                 Radius |
| ------- | ---------------------: |
| Buttons |                    5px |
| Inputs  |                    5px |
| Dialog  |                    8px |
| Tooltip |                    4px |
| Tabs    |                    4px |
| Cards   | Avoid unless necessary |

Avoid radius values above 10px unless implementing a modal or onboarding surface.

---

## 5. Component Rules

All UI components must be imported from `packages/ui`.

Do not create page-local variants of common components.

Shared components should include:

```text
Button
IconButton
Input
Textarea
Select
Checkbox
Switch
Tabs
TabBar
Toolbar
Sidebar
TreeView
DataTable
PropertyGrid
Dialog
Drawer
Popover
Tooltip
ContextMenu
DropdownMenu
CommandPalette
SplitPane
BottomPanel
StatusBadge
EmptyState
LoadingState
ErrorState
ConnectionStatus
```

Before creating a new component:

1. Search `packages/ui`.
2. Check whether an existing component can be extended.
3. Create a shared component only when the behavior is reusable.
4. Do not introduce page-specific styling for shared patterns.

---

## 6. Icon Rules

Use `lucide-react` only.

Recommended sizes:

| Context      | Size |
| ------------ | ---: |
| Toolbar      | 16px |
| Button       | 15px |
| Sidebar row  | 15px |
| Table action | 14px |
| Status bar   | 14px |

Rules:

* do not mix icon libraries
* do not use emojis as functional icons
* all icon-only buttons require tooltips
* use consistent stroke width
* avoid decorative icons without functional meaning

---

## 7. Visual Hierarchy

Each panel should contain no more than one primary action.

Examples:

* API Debugger: `Send`
* SQL Editor: `Run`
* Connection form: `Connect`
* Workspace creation: `Create workspace`

Secondary actions should use:

* icon buttons
* dropdown menus
* context menus
* command palette
* keyboard shortcuts

Avoid showing every action at the same visual priority.

---

## 8. State Handling

Every feature page must handle:

* default state
* empty state
* loading state
* error state
* disconnected state
* disabled state
* long text overflow
* narrow panel width
* keyboard focus state

A page is not complete if it only implements the happy path.

---

## 9. Interaction Rules

Prefer keyboard-friendly workflows.

Common actions should support:

* command palette
* keyboard shortcuts
* context menus
* tab navigation
* resizable panels
* collapsible panels
* persistent layout state

Use tooltips for icon-only actions.

Do not hide critical actions behind hover-only interactions.

---

## 10. Forbidden Patterns

Do not use:

* dashboard card grids for developer workflows
* excessive shadows
* random gradients
* oversized rounded corners
* oversized titles
* large marketing-style illustrations
* multiple icon libraries
* inconsistent toolbar heights
* inconsistent button styles
* module-specific visual languages
* unnecessary nested containers
* excessive borders between every section
* forms directly embedded into navigation trees
* page-local CSS for reusable patterns

---

## 11. Implementation Requirements

When implementing or refactoring UI:

1. Read this document first.
2. Inspect `packages/ui`.
3. Reuse shared layout components.
4. Use design tokens only.
5. Do not hardcode colors unless adding a token.
6. Do not create page-local button or input styles.
7. Add empty, loading and error states.
8. Verify narrow-width behavior.
9. Run frontend tests.
10. Generate screenshots for visual review.
