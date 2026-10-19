---
name: "Local Outlook"
description: "Local wayfinding for informed sales planning."
colors:
  teal: "#007d78"
  teal-dark: "#006964"
  teal-tint: "#e8f5f2"
  teal-bright: "#7de1ce"
  navy: "#142d42"
  navy-light: "#244255"
  rail-text: "#c3d4df"
  ground: "#f4f7f9"
  surface: "#fff"
  ink: "#183449"
  muted: "#5d7180"
  faint: "#e9eff3"
  line: "#dce5eb"
  amber: "#87531e"
  amber-tint: "#fff5e4"
  red: "#b33c3c"
  red-tint: "#fff0ef"
  cinema: "#72519a"
  cinema-tint: "#f3eef9"
  blue: "#3c6899"
  blue-tint: "#edf4fb"
  temperature-cold: "#244e92"
  temperature-cold-tint: "#e4edff"
  selection-soft: "#f3faf8"
typography:
  headline:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "36px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.028em"
  title:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "21px"
    fontWeight: 600
    lineHeight: 1.5
    letterSpacing: "-0.015em"
  subtitle:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.5
  body:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.5
  row-title:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "14px"
    fontWeight: 500
    lineHeight: 1.5
  label:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "12px"
    fontWeight: 500
    lineHeight: 1.5
  supporting:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.8
  date:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "26px"
    fontWeight: 500
    lineHeight: 1.3
rounded:
  compact: "6px"
  control: "7px"
  icon: "8px"
  row: "9px"
  surface: "12px"
spacing:
  base: "8px"
  compact: "12px"
  section: "16px"
  panel: "20px"
  roomy: "24px"
components:
  button-primary:
    backgroundColor: "{colors.teal}"
    textColor: "{colors.surface}"
    rounded: "{rounded.control}"
    padding: "11px 17px"
    height: "44px"
  button-primary-hover:
    backgroundColor: "{colors.teal-dark}"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "11px 17px"
    height: "44px"
  button-secondary-hover:
    backgroundColor: "{colors.ground}"
  button-danger:
    backgroundColor: "{colors.red}"
    textColor: "{colors.surface}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
    height: "40px"
  button-text:
    backgroundColor: "transparent"
    textColor: "{colors.teal}"
    padding: "4px"
    height: "36px"
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "10px 13px"
    height: "44px"
  navigation-item:
    backgroundColor: "transparent"
    textColor: "{colors.rail-text}"
    rounded: "{rounded.control}"
    padding: "12px 15px"
    height: "46px"
  navigation-item-active:
    backgroundColor: "{colors.navy-light}"
    textColor: "{colors.surface}"
  filter-selected:
    backgroundColor: "{colors.teal-tint}"
    textColor: "{colors.teal-dark}"
    rounded: "{rounded.compact}"
    padding: "7px 9px"
    height: "34px"
  source-panel:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.surface}"
    padding: "19px 20px 20px"
  signal-row:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.row}"
    padding: "14px"
  calendar-day-selected:
    backgroundColor: "{colors.selection-soft}"
    textColor: "{colors.ink}"
    padding: "10px 8px"
    height: "148px"
  calendar-popup:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.surface}"
    padding: "20px"
    width: "360px"
---
# Design System: Local Outlook

## Overview

**Creative North Star: "Local wayfinding"**

Local wayfinding gives the application a steady navy navigation rail, a pale blue working ground, and teal accents that guide action and selection. IBM Plex Sans, aligned rows, and restrained category colours support a practical desktop tool for reading local information. The identity remains generic across businesses.

The interface is calm and information rich. White surfaces and fine dividers organise the work without ornamental depth; dates, source labels, and compact disclosure rows establish the reading order. The daily agenda and calendar share the same category language, evidence rows, and source coverage cues.

**Key Characteristics:**
- Navy navigation with a bright teal active icon.
- Pale blue ground, white working surfaces, and fine cool dividers.
- IBM Plex Sans with medium-weight row titles and compact supporting text.
- Shared day markers, category icons, and explicit source states.
- Flat surfaces, short control feedback, and visible keyboard focus.

## Colors

The palette uses cool navy and pale blue for orientation, clear teal for interaction, and restrained colour pairs for source categories and operational states. Token values in the frontmatter match the CSS custom properties in `src/styles.css`.

### Primary

- **Wayfinding Teal** (`teal`): primary controls, links, focus outlines, active filtering, and fresh-source dots.
- **Deep Teal** (`teal-dark`): primary hover and readable text on teal-tinted selections.
- **Pale Teal** (`teal-tint`): selected filters, success messages, calendar selection, and local-event icon tiles.
- **Bright Teal** (`teal-bright`): the brand mark, active navigation icon, and focus outline against navy.

### Secondary

- **Warm Amber** (`amber`, `amber-tint`): school and bank holidays, stale or partial coverage, setup needs, and refresh reminders.
- **Cinema Violet** (`cinema`, `cinema-tint`): cinema-release labels and icon tiles.
- **Forecast Blue** (`blue`, `blue-tint`): weather labels, weather icons, and the preview notice.
- **Error Red** (`red`, `red-tint`): errors, failed tests, destructive action, and credential removal.

### Neutral

- **Navigation Navy** (`navy`): the navigation surface. **Raised Navy** (`navy-light`) marks hover and the current destination without a shadow.
- **Rail Mist** (`rail-text`): inactive navigation and secondary rail information.
- **Cool Ground** (`ground`): the application background and secondary hover surface.
- **White Surface** (`surface`): forms, rows, source panels, inputs, and on-colour control text.
- **Ink Navy** (`ink`): headings, row titles, and primary body text.
- **Slate** (`muted`): metadata, secondary copy, inactive controls, and unavailable dates.
- **Pale Divider** (`faint`): subtle separators and unavailable-coverage backgrounds.
- **Cool Line** (`line`): control strokes, card borders, calendar grid, and section dividers.

**The Wayfinding Rule.** Use teal for primary action, selection, links, and positive source state; keep category colour confined to the relevant icon, label, or coverage message.

**The Labelled State Rule.** Pair status colour with a readable state label or icon so freshness and failure remain understandable without colour.

## Typography

**Display Font:** IBM Plex Sans, with sans-serif fallback.
**Body Font:** IBM Plex Sans, with sans-serif fallback.

**Character:** One humanist sans-serif family keeps the tool direct and consistent across platforms. The bundled regular, medium, and semibold weights carry the hierarchy; headings are compact, while explanatory copy has more leading.

### Hierarchy

- **Headline** (`headline`): the page title. It reduces to (32px) at the tablet breakpoint and (29px) at the narrow breakpoint.
- **Title** (`title`): agenda, calendar month, and section headings.
- **Subtitle** (`subtitle`): subsection headings. Settings headings use local (18px) and (17px) variants.
- **Body** (`body`): the application base; screen-specific introductions commonly use (14px).
- **Row Title** (`row-title`): factor names, navigation, and button labels. Factor names reduce to (13px) on narrow screens.
- **Label** (`label`): short field labels, filters, state labels, and compact controls.
- **Supporting** (`supporting`): evidence, setup notes, and coverage explanations. Longer explanatory lines have observed maximum widths between (65ch) and (85ch); compact panel copy is narrower.
- **Date** (`date`): the shared day number, paired with an uppercase (12px) weekday at (0.06em) tracking. Dates and calendar columns use tabular numerals.

**The Quiet Hierarchy Rule.** Use weight, size, alignment, and spacing to establish hierarchy. Keep supporting copy subordinate and preserve the title–metadata relationship in every factor row.

## Layout

The desktop shell has a sticky, full-height navigation rail (208px) and a flexible workspace capped at (1600px). The standard workspace inset is (44px 42px 22px). Its form sits above the main reading area; the latter pairs flexible content with a source panel (285px) and a (27px) gap. Settings use a narrower form capped at (1000px).

Spacing is compact and aligned rather than mathematically uniform. The declared base space is (8px); repeated working gaps use (12px), (16px), (20px), and (24px). Keep the source's component-specific insets rather than forcing every distance onto one grid. Agenda groups use a date column (45px), a content column, and a (16px) gap; factor rows within the day are separated by (7px).

At large widths (1500px and above), workspace insets and source panel width increase. At (1150px and below), the rail narrows to (188px) and working gaps tighten. At (1000px and below), source information moves below the agenda. At (800px and below), the rail becomes a horizontal top navigation, secondary rail content disappears, and the workspace inset contracts. At (560px and below), the postcode form wraps, settings fields stack, the date column narrows to (35px), and the workspace uses (18px) side insets. The supported page minimum is (360px).

The calendar spans the workspace width, with source information below. It uses seven columns and only the four to six weeks required by the month. Fully unavailable weeks compact to 64px; loaded weeks have 148px minimum cells (138px on narrow screens). Source rows use three columns below the calendar, reducing to two at (800px) and one at (560px). Individual badges and weather conditions open anchored, non-modal detail popups. The date control and “+N more” open a day list in the same popup; details are no longer below the grid. View controls wrap on narrow screens, preserving the month, selected date, and cached-data context.

## Elevation & Depth

Working surfaces are flat. The floating calendar popup alone uses an offset soft shadow to distinguish its temporary layer from the grid beneath. White surfaces, pale ground, subtle strokes, and navy selection tones convey grouping. The sticky navigation and settings save bar establish persistence through position and borders. Agenda disclosures expand inside their rows. Calendar popups use a white surface, 12px corners, a small anchor arrow, and `0 12px 36px rgba(20,45,66,.18)` shadow without an additional border.

**The Flat Surface Rule.** Separate working areas with background tone and fine borders. Keep cards and controls free of decorative shadows.

## Shapes

Working containers use gently rounded corners (`surface`); controls use the tighter `control` radius. Factor rows use `row`, icon tiles use `icon`, and compact filters and coverage notices use `compact`. Fine one-pixel strokes define boundaries. The calendar grid keeps square shared edges, with a two-pixel teal border inside the selected cell. Circular shapes identify today in the calendar as well as status dots, the empty-state compass, and switch thumbs.

Icons are simple outlined SVGs with a shared stroke weight (1.7), generally (15–22px). Larger icons belong to the brand mark or empty-state illustration. Place category icons inside tinted square tiles rather than letting category colour spread across the entire row.

## Components

### Buttons

Compact and practical. Primary actions use teal with white text, the `control` radius, and the `button-primary` inset. Standard controls have a minimum height (44px); the compact variant uses (40px) and (8px 12px) insets. The postcode action aligns to its field at (47px).

Secondary actions are white with a cool-line stroke; hover uses cool ground with a slate border. Primary hover darkens teal. Text actions use teal and underline on hover. Destructive confirmation uses red. Disabled buttons reduce opacity to (.5). Pressing an enabled button moves it down (1px).

State transitions are short (160ms) and affect backgrounds, borders, or colour. Keyboard focus uses a teal outline (3px) with an offset (3px); the rail uses bright teal. Reduced-motion preference removes transitions and spinner animation.

### Chips

Category filters are compact text buttons with the `compact` radius. The selected filter uses pale teal, deep teal text, and medium weight; inactive filters are transparent with slate text and gain pale-divider background on hover. The agenda/calendar view switch repeats this selection language inside one bordered control.

### Cards / Containers

White, finely outlined, and flat. Source panels, settings sections, and the postcode form use the `surface` radius. The source panel uses the `source-panel` inset; settings sections use (26px) on desktop and tighter insets as space contracts. Factor rows use the smaller `row` radius and remain visually subordinate to the overall section.

### Inputs / Fields

White fields use cool-line strokes, the `control` radius, and the `input` inset, with a minimum height (44px). Focus changes the stroke to teal and retains the visible outline. Place labels above fields, using a clear gap; place explanation and status below. The postcode field is larger (16px), includes a location SVG, and renders entered text in uppercase. Search within loaded results is a quieter, borderless field above a section divider.

### Navigation

Navy anchors the interface. Destinations use slate-blue rail text and medium-weight labels; the current destination and hover share the lighter navy fill. Current destination text is white, with a bright teal SVG. Navigation becomes horizontal at the tablet breakpoint and keeps its selected state. The product mark uses a compass and a two-line wordmark.

### Daily agenda and evidence rows

The shared date marker is the signature pattern: weekday, large day number, and month align beside all of that day's factors. A factor row has a category tile, a readable title, muted metadata, and a disclosure chevron. Opening it reveals description, provider evidence, and source links without changing the row's visual identity. Calendar popup details retain the same category colour language.

### Source states and coverage

Fresh source state is teal; partial, stale, or setup-needed state is amber; error is red; disabled is neutral. A small dot and a readable label carry state together. Source rows disclose coverage details and next actions. Success and error messages pair tinted backgrounds with icons and readable feedback. Days beyond the loaded forecast say “No forecast”; opening their day popup also states the last loaded forecast date.

### Calendar

Each cell starts with the date on the left and the forecast-high temperature badge on the right. Below that header, the factor list starts with the weather condition and its matching SVG icon, followed by the event and holiday badges. Weather remains visible when filtering other factors. Badge bands use cold blue (below 6°C), forecast blue (6–11°C), teal (12–19°C), amber (20–25°C), and red (26°C and above). The full range and conditions remain in the tooltip, accessible name, and weather details. The selected cell uses the softer `selection-soft` tint, keeping entry and weather badges distinct. The grid has a 700px minimum width and scrolls within its own container when space is tighter, keeping day numbers and badges legible.

Individual events, film releases and holidays appear as 12px named badges with category icons, 6px padding and 6px corners. Teal, violet and amber backgrounds separate the categories. Three badges fit before a readable “+N more” button. Each badge is independently keyboard accessible and opens its complete evidence in a popup; the date and overflow buttons open the complete filtered day list. Long names truncate visually and retain full text in tooltips and the day’s accessible name. Today uses a teal circle around its number.

Dates show compact factor badges using the same category colours. The selected date uses the `selection-soft` fill and a two-pixel teal stroke. Unavailable dates are pale and disabled; other-month numbers are muted. Hover uses pale teal and keyboard focus remains explicit. Arrow keys move through dates in the loaded range. Popups stay within the viewport and follow their anchor on scrolling or resizing. Escape and the close button return focus to the trigger; outside click or focus moving away dismisses without stealing focus. Popup actions read cached data and make no API calls.


### Calendar popup

The temporary detail surface uses the `calendar-popup` tokens and the calendar-popup shadow recorded in the sidecar. Its white, borderless surface has (12px) corners and a small rotated-square caret (12px). The panel is (360px) wide, capped to the viewport with (12px) margins; its scrollable interior uses (20px) padding and a maximum height of (560px), further bounded by viewport height. Placement prefers the trigger's right side, then left, then above or below, with a (12px) anchor gap.

A named, non-modal dialog contains a wrapping heading, a close button, category label, labelled date/source metadata, description, and a source link where available. Day-list popups put matching weather first and open individual evidence in the same panel. Opening focuses the close button; the trigger remains the point of return on Escape or close. The popup follows scroll and resize, and dismisses if its anchor leaves view.

### App updates

App updates uses the existing white Settings section, compact secondary check action, teal install/restart actions, and inline status text. Release notes open in a disclosure as literal, scrollable text. Download progress is determinate when the total is known and indeterminate otherwise. Checking, installing, and restarting temporarily disable source requests and settings changes while cached browsing remains available; Overview shows a status message with a “View update” action. Unsaved settings or active source work block updating. Installation and restart each require an explicit action, with retry and quit/reopen recovery explained inline.

## Do's and Don'ts

### Do:

- **Do** retain the navy navigation, pale working ground, and teal action language.
- **Do** group related factors under one day marker; use disclosures in the agenda and anchored popups in the calendar.
- **Do** use the existing category icon and colour pairing in both agenda and calendar.
- **Do** show source freshness, unavailable coverage, and action feedback in readable text.
- **Do** preserve the visible focus ring and the reduced-motion behaviour.
- **Do** reflow the rail to the top and stack source information as space narrows.

### Don't:

- **Don't** apply a restaurant-specific brand to this generic product.
- **Don't** use status or category colour as the only explanation of a state.
- **Don't** add decorative shadows to the flat surface hierarchy.
- **Don't** hide coverage limits behind an empty date or an apparently complete list.
- **Don't** shrink the reusable body or label hierarchy to fit a dense new surface.
