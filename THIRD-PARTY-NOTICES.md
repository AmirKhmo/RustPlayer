# Third-party notices

The RustPlayer source code is MIT licensed (see `LICENSE`).
The binary releases bundle the following third-party components, each under its own license:

| Component | License | Source |
|---|---|---|
| libmpv (shinchiro Windows build) | GPL-2.0-or-later (this build includes GPL parts) | https://github.com/mpv-player/mpv · https://github.com/shinchiro/mpv-winbuild-cmake |
| FFmpeg, libass and other libraries inside libmpv | LGPL / GPL / various | see the mpv-winbuild-cmake repository |
| Vazirmatn font | SIL Open Font License 1.1 | https://github.com/rastikerdar/vazirmatn |
| Rust crates (egui, eframe, libmpv2, rfd, ...) | MIT / Apache-2.0 | https://crates.io |

Because `libmpv-2.dll` is GPL-licensed, the **combined binary distribution** (portable zip and installer)
is covered by the GPL. The complete corresponding source of RustPlayer is this repository; the source of
libmpv and its dependencies is available at the links above.
