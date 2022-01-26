# 2.0.999 (Second Beta) (2022-01-26)

## Features

- Added News Panel
- Added Device Simulation Panel
- Rebuilt Server Status GUI to use grid layout
- Change panel storage model to persist through app lifetime

# 1.0.999 (First Beta) (2022-01-04)

## Features

- Add status icons to left side of server status for updates, error messages, etc.
- Add disconnect button to device test panel
- Add start server on startup option
- Add check for updates on startup option
- Turn server start button to X if no engine present
- Fill out more of first run experience
- Refactor update system, can now update engine/device file and warn user to update application by
  hand
- Add tooltips to buttons that may not be obvious otherwise

# Alphas 1-4 (2021-12-28)

## Features

- Reimplementation of the Electron/Typescript Intiface Desktop in Rust/egui
- Runs engine process
- Has simple device interaction implementation
- Usual settings from old Intiface desktop
- Crash logging and log submission via Sentry
- New 2-mode UI: Collapsed/Extended. Thanks, Winamp.