# deadlocked

a very simple cs2 aimbot, for linux only.

## setup

- add your user to the `input` group: `sudo usermod -aG input username`
- restart your machine (this will not work without a restart!)

## running

- if you got the source code from github, run with cargo: `cargo run --release`
- if you got a standalone binary, just run that

## security

- should be "undetectable", as far as user-space externals go
- completely in user-space
- does not write anything to game memory
- does not need root permissions
