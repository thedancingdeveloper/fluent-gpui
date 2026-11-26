# The gallery as a regression matrix

The gallery (`cargo run -p gallery`) groups its pages by Fluent 2 component
family: buttons and links, selection controls, text and labels, status and
feedback, data and lists, date and input, terminal and video, and theme and
direction. Pick a family in the left sidebar. Each page shows the state
variants of its controls side by side (default, selected, disabled,
indeterminate, error). The variant row on every page switches between light,
dark, high contrast and right-to-left, so one pass through the families covers
every state in every variant.

After a change to a primitive or to theme tokens, walk the families in all four
variants and compare them with [VISUAL_TARGETS.md](VISUAL_TARGETS.md). A new
public primitive gets its states added to the matching family page in
`examples/gallery/src/families.rs`.

`gallery --list-pages` prints the page ids. The [screenshots harness](../CONTRIBUTING.md#screenshots)
renders every page headlessly.
