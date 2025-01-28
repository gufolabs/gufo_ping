# Supported Platorms

## Binary Wheels

Following binary wheels are provided with latest release:

| OS                       | Arch    | Libc                  | 3.8                        | 3.9                        | 3.10                       | 3.11                       |
| ------------------------ | ------- | --------------------- | -------------------------- | -------------------------- | -------------------------- | -------------------------- |
| :simple-linux: Linux     | x86_64  | Glibc 2.17 (RHEL 7)   | :material-check-bold:      | :material-check-bold:      | :material-check-bold:      | :material-close-thick:[^1] |
| :simple-linux: Linux     | x86_64  | Glibc 2.24 (Debian 9) | :material-check-bold:      | :material-check-bold:      | :material-check-bold:      | :material-check-bold:      |
| :simple-linux: Linux     | aarch64 | Glibc 2.24 (Debian 9) | :material-close-thick:[^2] | :material-close-thick:[^2] | :material-close-thick:[^2] | :material-close-thick:[^2] |
| :simple-linux: Linux     | x86_64  | Musl 1.1              | :material-check-bold:      | :material-check-bold:      | :material-check-bold:      | :material-close-thick:[^3] |
| :simple-linux: Linux     | aarch64 | Musl 1.1              | :material-close-thick:[^4] | :material-close-thick:[^4] | :material-close-thick:[^4] | :material-close-thick:[^4] |
| :simple-macos: MacOS     | x86_64  |                       | :material-check-bold:      | :material-check-bold:      | :material-check-bold:      | :material-check-bold:      |
| :simple-macos: MacOS     | arm64   |                       | :material-close-thick:[^5] | :material-close-thick:[^5] | :material-close-thick:[^5] | :material-close-thick:[^5] |
| :simple-windows: Windows | x86_64  |                       | :material-close-thick:[^6] | :material-close-thick:[^6] | :material-close-thick:[^6] | :material-close-thick:[^6] |
| :simple-freebsd: FreeBSD | x86_64  |                       | :material-close-thick:[^7] | :material-close-thick:[^7] | :material-close-thick:[^7] | :material-close-thick:[^7] |

[^1]: Python 3.11 is not supplied with `manylinux2014` image.
[^2]: `manylinux_2_24_aarch64` build failed. Rustc exists with code `-9`.
[^3]:
    auditwheel stops with:
    ValueError: Cannot repair wheel, because required library `librt.so.1` could not be located
[^4]: `musllinux_1_1_aarch64` build failed. Rustc exists with code `-9`.
[^5]: Failed to build on ARM64 platform.
[^6]: Not supported yet. Needs separate implemetation due to different raw sockets API.
[^7]: Volonteers are needed.