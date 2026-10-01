---
name: "Tactica"
description: "An approachable, understated workspace for milsim unit management."
colors:
  background: "#f5f4f7"
  foreground: "#252329"
  card: "#fdfcfe"
  card-foreground: "#252329"
  popover: "#fdfcfe"
  popover-foreground: "#252329"
  primary: "#a78bfa"
  primary-foreground: "#17121f"
  secondary: "#eee8fc"
  secondary-foreground: "#41335f"
  muted: "#eee8fc"
  muted-foreground: "#696372"
  accent: "#eee8fc"
  accent-foreground: "#41335f"
  destructive: "#9c382f"
  border: "#e1dde7"
  input: "#c4bdcf"
  ring: "#7955c8"
  sidebar: "#19171c"
  sidebar-foreground: "#f5f2fb"
  sidebar-primary: "#a78bfa"
  sidebar-primary-foreground: "#19171c"
  sidebar-accent: "#a78bfa"
  sidebar-accent-foreground: "#17121f"
  sidebar-border: "#39333f"
  sidebar-ring: "#a78bfa"
  brand-ink: "#6742b5"
  surface: "#fdfcfe"
  selection: "#eee8fc"
  sidebar-muted: "#b5adbf"
typography:
  headline:
    fontFamily: "Geist Variable, sans-serif"
    fontSize: "30px"
    fontWeight: 650
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  title:
    fontFamily: "Geist Variable, sans-serif"
    fontSize: "18px"
    fontWeight: 600
    lineHeight: 1.35
  body:
    fontFamily: "Geist Variable, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Geist Variable, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  sm: "2.4px"
  md: "3.2px"
  lg: "4px"
  xl: "5.6px"
  pill: "999px"
spacing:
  "2": "8px"
  "3": "12px"
  "4": "16px"
  "5": "20px"
  "6": "24px"
  "9": "36px"
components:
  button-primary:
    textColor: "{colors.primary-foreground}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 10px"
    height: "32px"
    backgroundColor: "{colors.primary}"
  button-outline:
    textColor: "{colors.foreground}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 10px"
    height: "32px"
    backgroundColor: "{colors.background}"
  button-secondary:
    textColor: "{colors.secondary-foreground}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 10px"
    height: "32px"
    backgroundColor: "{colors.secondary}"
  button-ghost:
    textColor: "{colors.foreground}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 10px"
    height: "32px"
  button-destructive:
    textColor: "{colors.destructive}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 10px"
    height: "32px"
  button-link:
    textColor: "{colors.primary}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "0 10px"
    height: "32px"
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.foreground}"
    typography: "{typography.body}"
    rounded: "{rounded.lg}"
    padding: "9px 12px"
  role-pill:
    backgroundColor: "{colors.selection}"
    textColor: "{colors.brand-ink}"
    rounded: "{rounded.pill}"
    padding: "3px 9px"
  navigation-active:
    backgroundColor: "{colors.sidebar-accent}"
    textColor: "{colors.sidebar-accent-foreground}"
    height: "40px"
  form-panel:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.lg}"
    padding: "24px"
---

# Design System: Tactica

## Overview

**Creative North Star: "The Unit Workspace"**

Tactica is a practical, calm, organised workspace. Its approachable and understated character comes from soft lavender accents, pale working surfaces, and a dark navigation rail. The interface gives unit members and administrators clear places to read, select, and edit.

Compact controls and restrained corners support dense administrative work. Borders and tonal changes establish groups; elevation signals temporary layers or movement. Preserve this quiet character when extending the interface.

**Key Characteristics:**

- Soft Lavender actions and selected navigation against Charcoal Plum.
- Geist typography, compact controls, and clearly separated content groups.
- Flat by default, with functional elevation.
- Approachable and understated buttons, fields, and panels.

This is a scan of the implemented workspace, with descriptive language confirmed by the user. Sources: `web/src/app.css`, `web/src/global.css`, `web/src/components/ui/`, and the application components. `main.tsx` imports the global stylesheet before the application stylesheet; unlayered application rules override the starter theme and some utility styles. Desktop (1440px) and mobile (390px) role views were checked in Chromium. No PRODUCT.md exists. The frontmatter records current reusable values, not a proposed redesign; spacing names reflect existing utility steps, not a complete enforced scale.

## Colors

A lavender accent warms pale neutral surfaces while Charcoal Plum anchors navigation. Frontmatter is the normative value list; aliases reflect the actual CSS roles.

### Primary

- **Soft Lavender** (`primary`, `sidebar-primary`, `sidebar-accent`): primary actions and selected, hovered, or focused sidebar entries. Pair with their dark foreground tokens.
- **Violet Ink** (`brand-ink`): role labels, smaller accent details, and text-entry carets.
- **Focus Violet** (`ring`): visible keyboard focus on the light workspace.

### Neutral

- **Pale Workspace** (`background`): the overall content canvas.
- **Paper Surface** (`card`, `popover`, `surface`): forms, tables, and temporary surfaces; use their matching foreground roles.
- **Charcoal Text** (`foreground`): primary content text.
- **Lavender Wash** (`secondary`, `muted`, `accent`, `selection`): selected rows, subtle grouping, and secondary actions. Their foreground roles distinguish emphasis from supporting text.
- **Muted Plum Grey** (`muted-foreground`): descriptions and supporting information.
- **Soft Divider** (`border`) and **Field Border** (`input`): separate groups and define editable fields.
- **Charcoal Plum** (`sidebar`): the persistent navigation surface, with pale `sidebar-foreground`, subdued `sidebar-muted`, and `sidebar-border` dividers.

**Destructive Brick** (`destructive`) identifies destructive controls and errors. It is a semantic status colour, not a second brand accent. The repository includes starter `.dark` rules, but the verified workspace uses the application palette; do not infer a completed dark theme from those rules.

## Typography

**Workspace font:** Geist Variable, sans-serif. Body, headings, controls, and navigation share this family.

**Character:** clear and understated, with weight and spacing establishing hierarchy rather than contrasting display faces.

### Hierarchy

- **Headline:** page titles use `typography.headline`; mobile titles reduce to (26px) at widths up to (600px).
- **Title:** section headings use `typography.title`. Third-level headings use (16px), weight (600).
- **Body:** descriptions and editable content use `typography.body`.
- **Label:** metadata commonly uses `typography.label`; helper text and role pills use (11px), sidebar entries (13px). These are contextual sizes, not one universal label style.

The documentation-only `.typeset-docs` scope declares Nunito Sans for body, Space Grotesk for headings, and Geist Mono for code. Keep that separate from workspace typography. The unlayered `font: inherit` control rule makes rendered standard buttons regular weight, despite the component’s `font-medium` utility.

## Layout

The desktop shell uses a navigation rail (16rem), a top bar (62px), and a centred content region with maximum width (1500px). Default content padding is (38px 36px 60px). The mobile navigation sheet uses (18rem); the sidebar switches at the (768px) breakpoint.

At widths up to (1100px), content padding becomes (32px 26px 50px). Up to (800px), it becomes (30px 22px 40px), and the top bar becomes (58px). Up to (600px), padding becomes (28px 18px 40px), headings and actions stack, search fills its container, two-column forms become one column, and the role list sits above its editor.

The desktop role layout starts with a (270px) list column beside flexible content, reducing that column to (220px) at the intermediate breakpoints. Tables retain their tabular structure inside an overflow container. Metadata uses tabular numerals where comparison matters. Repeated spacing steps are recorded in frontmatter; component-specific gaps and insets remain intentional exceptions.

## Elevation & Depth

Flat by default, with functional elevation. Ordinary panels use pale surfaces and thin borders. Sheets and menus can lift above the workspace, while a dragged role uses the explicit shadow (`0 8px 20px #19171c26`) with a focus-coloured outline. Dialog overlays use a faint black scrim and supported backdrop blur; dialog surfaces use a thin ring. These treatments distinguish temporary layers from persistent content.

**The Functional Elevation Rule.** Use elevation to communicate a temporary layer or a moving item; retain flat resting panels.

## Shapes

The base radius is `rounded.lg`; utility small, medium, and extra-large corners derive from it. Fields, table containers, and most panels are gently squared. Role pills use `rounded.pill`. Retain these contextual differences rather than rounding every component alike. Thin borders establish boundaries without heavy frames.

## Components

Components are **approachable and understated**. Frontmatter records compact defaults; state styling, motion, and illustrative snippets live in `.impeccable/design.json`.

### Buttons

The shared component provides primary (its API calls this `default`), outline, secondary, ghost, destructive, and link variants. Standard controls are (32px) tall with horizontal padding (10px); icon and size variants exist. At widths up to (600px), application buttons have a minimum height of (40px).

Primary hover reduces the accent background to (80%) opacity. Outline and ghost hover use the muted surface; secondary hover mixes its background with foreground (5%). Destructive buttons use a (10%) destructive tint, rising to (20%) on hover. Focus uses a ring and border treatment. Pressed buttons move down (1px), except popup triggers; disabled buttons suppress interaction and reduce opacity. Legacy `.button` links use their own (36px) minimum height and (8px 14px) padding.

### Inputs / Fields

Inputs use a surface fill, field border, base radius, and minimum height (40px), with padding from `components.input`. Textareas start at (90px) and resize vertically. The application CSS overrides some input utilities, so copy the rendered treatment rather than assuming utility defaults. Focus has a violet outline and the component ring; search containers use `:focus-within`. Read-only and disabled fields have distinct pale fills; invalid fields use destructive borders and rings.

### Navigation

The Charcoal Plum rail contains unit switching, workspace links, administration disclosure, and account controls. Main entries are (40px) tall, use (13px) type, and turn Soft Lavender on hover, focus, or selection. Their text and icons switch to the dark accent foreground together. Administration sublinks follow the same colour logic. On mobile, the rail becomes a dismissible sheet.

### Role pills

Small wrapping labels use a lavender wash and Violet Ink, capsule corners, and padding from `components.role-pill`. They cap their width at (24ch), wrap long names, and retain text labels. Linked pills underline on hover and show a visible focus outline.

### Panels and tables

Form panels use a thin divider border, pale surface, and base radius; the create-unit panel uses `components.form-panel` padding. Other panels preserve their own context-specific insets. Roster headers use (12px) text and (14px 20px) padding; body cells use (15px 20px), with a (68px) row height. On small screens, cell padding becomes (13px 12px).

### Role list and editor

A selected role uses a lavender wash within the bordered list. Dragging adds functional elevation and an outline. Mobile drag handles expand to (44px); a Reorder control exposes the interaction. Preserve the distinct locked, selected, and draggable states and their textual explanations.

### Motion

Administration chevrons rotate over (150ms ease); sidebar layout transitions use (200ms linear), sheets (200ms ease-in-out), and dialog overlays (100ms). The reduced-motion media query disables animations and transitions throughout the application.

## Do's and Don'ts

### Do:

- **Do** reuse the application semantic colour variables and existing component variants.
- **Do** preserve visible focus, readable disabled states, and reduced-motion handling.
- **Do** stack forms and role layouts at the existing mobile breakpoints.
- **Do** reserve rounded pills for compact role labels; keep panels and fields gently squared.

### Don't:

- **Don't** treat the starter dark-theme declarations as a verified alternate Tactica theme.
- **Don't** substitute documentation typefaces for the workspace’s Geist typography.
- **Don't** add permanent shadows to ordinary panels or remove the distinct dragging treatment.
- **Don't** encode role meaning solely through a colour; retain its text label.
