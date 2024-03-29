# v0.41.0

Improvements:

- Async callbacks
- Tabs in custom layouts
- `TracerAction` and `meio-addon` to use tracers with `meio` actors

# v0.40.0

Improvements:

- Support config file
- Fully custom layouts (cases)
- `LiveTail` tracer
- Flexible width of `Table` and `LiveTail`


# v0.39.0

Improvements:

- New `LiveText` tracer with **Markdown** rendering
- New `Alert` tracer and rendering as popup notification
- New layout based of `flex`
- Improved layout for mobile devices


# v0.38.0

Improvements:

- `Range` use `Bound` with `strict`, `loose` or `auto` limit
- Prime Pack moved to the `prime` submodule
- Add internal layers: Visual, Control, Transparent
- Group controls in a separate panel
- Add options to all tracers


# v0.37.0

Improvements:

- New: `Gauge` tracer that rendered as a progress bar
- Refator: names of the existent tracers reduced to: `Board`, `Counter` and `Pulse`
- Method `start` stores rillrate's handle globally
- Add `stop` method for termination the rillrate engine
- Controls added:
    - `Click` control
    - `Switch` control
    - `Selector` control
    - `Slider` control
- Data flows added:
    - `Gauge`
    - `Histogram`
    - `Table`
- Strings as paths allowed. For example: "package.dashboard.group.my-data-flow"
- Python support

Fixes:

- UI: The dashboard gets stuck if pulse frame is empty
- UI: The dashboard didn't update a card if it has the same `EntryId`
- UI: Add scrolling to the `Board` tracer
