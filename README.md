# Trail Carrier

**Collect every parcel and reach the exit within the move budget.**

A native game for PrimitiveOS. Tap a neighboring square or use an arrow to move one step. Each parcel makes your trail longer. Plan a path around walls and your trail, then head for the exit when it opens.

## How to play

- **Step**. Tap a neighboring square or use the arrow buttons. A blocked step costs no move.
- **Collect parcels**. A parcel grows your trail. On a regular step, the tail moves off its last square.
- **Finish the route**. The exit opens after you collect every parcel. Clear ten routes and use Undo to recover from a wrong turn.

Progress saves between launches. **Done** returns to Launcher.

## Screenshots

<table>
<tr>
<td align="center"><img src="docs/images/trailcarrier.png" width="250" alt="The first route with a courier, parcel, walls and locked exit"><br><sub>The first route with a courier, parcel, walls and locked exit</sub></td>
<td align="center"><img src="docs/images/trailcarrier-playing-dark.png" width="250" alt="A longer trail and an open exit in dark mode"><br><sub>A longer trail and an open exit in dark mode</sub></td>
<td align="center"><img src="docs/images/trailcarrier-complete.png" width="250" alt="A completed delivery with one move left"><br><sub>A completed delivery with one move left</sub></td>
</tr>
</table>

Desktop simulation captures at 1264 × 1680 pixels.

## Install

After setting up PrimitiveOS, open **Store** from Launcher. Find **Trail Carrier** under **Apps** and choose **Get**.
See the [installation guide](INSTALL.md) for the local build workflow.

## Develop locally

Keep the Tools and Launcher checkouts next to this repository.

```sh
cargo test --offline --locked --workspace
cargo run --offline --locked -p trailcarrier-desktop
```

The desktop target exercises the touch controls and writes previews to `build/previews/`.

## Licence

Copyright (C) 2026 Doan Nguyen.

Code is licensed under [PolyForm Noncommercial 1.0.0](LICENSE). The documentation and original art are
licensed under [CC BY-NC-SA 4.0](LICENSE-DOCS).
