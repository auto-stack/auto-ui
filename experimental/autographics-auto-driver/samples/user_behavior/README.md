# Portable user behaviors

Copy either complete move or patrol directory outside this repository, including
its assets. Start the same autographics-authoring executable with
`--open <directory>/scene.agscene --behavior main.at`, then choose Run. A different
process working directory is supported. Move uses arrows/V in either focused
canvas. Patrol moves the 3D parent automatically and derives the 2D vertical
position from the child world position; arrows/V control its 2D actor.

Stop restores the Editor. Edit only main.at and Run again: globals reset and the
new file bytes are snapshotted. Changing Move speed from60.0 to90.0 requires no
runner/bridge/Rust changes. Relocating the complete directory keeps main.at and
relative asset resolution valid. Save As changes the scene root, so copy the
script/assets as needed; source selection is session state and is not saved.

These are independent, ordinary callbacks using the public behavior/1 API;
no transport functions or embedded profile6 string are needed. See
../../api/README.md for signatures, bounds and failure semantics.
