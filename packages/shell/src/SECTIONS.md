# The GM shell's sections and who builds them

Alex, 24 Nov: one builder per section of the screen, each in its own folder, with anything shared behind a small
agreed interface. `App.svelte` is the shared shell: it holds the session, the workspace, where the GM is, the desks,
lenses and depth, the campaign file, and it passes each section what it needs as props and callbacks. A section does
not reach into App; App does not lay out a section's insides.

| Folder | Section | Builder thread |
|---|---|---|
| `nav/` | The place line (`PlaceLine.svelte`: name, steps, the picker and going down), the instrument rail (`Rail.svelte`), the object menu | Build object selector and menus |
| `chart/` | The stage (`Stage.svelte`: the view in front, view controls, tools, notes, the card), the system chart and body cards | Make the SSE4 test drive work end to end (chart room) |
| `run/` | The table on the status line (`TableStatus.svelte`: on-air chip off the Run desk, Send this view, players' link); the Switchboard, Playbook, Bookmarks and the role card (`RoleCard.svelte`, PB7, which replaced the Seats dialog); the players' page (`Players.svelte`, `players.ts`, `share.svelte.ts`) is theirs too | Build Run desk and players' side |
| (App and the rest) | Shared shell: bar, desks, lenses, depth, More, the ruler footer, dialogs, the campaign file | Ask in the project before changing; one small PR at a time |

Editing (Build desk tools beyond the stage's) gets its own folder and builder once "Design planet and star editing"
is accepted.

A change that crosses sections goes through App's props in one PR, with a note to each section's builder.
