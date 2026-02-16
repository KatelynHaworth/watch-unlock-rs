# ⌚ Apple Watch Unlock

> **NOTE**: This repository is in a pre-release stage. See TODO list bellow for `v1.0.0` requirements

This repository hosts a PAM module, and CLI tool for configuring it, that allows
for the unlocking of a Linux system using the presence of an Apple Watch.

Each user on the system can be configured to be unlocked using their own personal Apple Watch.

## How it works

The PAM module uses Bluetooth Low Energy to detect the presence of an Apple Watch by using an
[Identity Resolution Key][0] to match the resolvable random address of that specific Apple Watch.

When the Apple Watch is detected, the received siginal strength indicator is used to determine if
the watch is "close enough". If the watch is close enough, the [Apple Continuity - Nearby Info][1]
of the watch is interigated to check that it is both unlocked _and_ the watch has been configured
to automatically unlock supporting devices.

## Installing

### Manual

To install manually you will first need to install a Rust toolchain (e.g. [rustup][2]), and also make
sure that a Bluetooth stack is installed (e.g. `bluez`)

```bash
git clone https://github.com/KatelynHaworth/watch-unlock-rs
cd watch-unlock-rs
make
sudo make install
```

### Arch Linux

> **TODO:** A `PKGBUILD` is available in this repository but has not yet been submitted to the AUR.

A version of this PAM module is available via [AUR][3] and can be installed using a tool like `yay`.

```bash
yay -S watch-unlock-rs
```

## Configuring

### Add your user to the PAM module configuration

> Check the [ESPresence Apple Guide][4] on how to obtain the Identity Resolution Key for your Apple Watch

Adding a new user, and associated Apple Watch, is made easy by using the included `watch_unlock_cli` tool,

```bash
sudo watch_unlock_cli add_user [username] [identity_resolution_key]
# Example: watch_unlock_cli add_user admin XkVgPxNEK0p4TDgZegzDUA==
```

It is also possible to manually configure a user-watch association by directly modifying the PAM module configuration.

```bash
sudo vim /etc/security/apple_watch.conf
```

### Testing the new user association

Before configuring your desired PAM policies, it is best to first check that the PAM module can validate a user against
the configured Identity Resolution Key.

```bash
watch_unlock_cli pam_test [username]
# Example: watch_unlock_cli pam_test admin
#> Connecting to Apple Watch PAM module [apple-watch]
#> Testing PAM module authentication with user 'katelyn'
#> Decoding Identity Resolution Key for Apple Watch
#> [apple-watch] INFO: Searching for Apple Watch
#> Found Apple Watch after 1 tries
#> [apple-watch] INFO: Unlocking with Apple Watch
#> Authentication was successful!
```

### Enable auto-unlock for lock screens

> This example is for KDE Plasma but can be applied to all other PAM policies you may want to use it for

The greatest use of this PAM module is the ability automatically unlock your session when it has gone to sleep, to do
this simply configure the `kde` PAM policy to first use the Apple Watch PAM module and then fall back on the default
`system-login` policy.

```bash
sudo vim /etc/pam.d/kde

#%PAM-1.0 
 
auth    include apple-watch 
auth    include system-login
```

## References

A ***huge*** shout to [DavidSt49/watch-unlock-linux][5] for being a massive inspiration for this project and being a
great
reference for implementation details on detecting and interrogating Apple Watches.

## TODO Before `v1.0.0`

Before a release will be marked as `v1.0.0` the following needs to be completed

- [ ] Make the unlock threshold configurable in PAM policy
- [ ] Replace usages of `println!` and `eprintln` in the PAM module with syslog
- [ ] Scrutinise rust dependencies to remove waste (LTO is already enabled, but anything to shorten build times)
- [ ] Upstream PAM module side conversation implementation to the `pam` crate
- [ ] (Nice to have) Update the PAM client (in the `pam` crate) to allow passing a username when the client is created
- [ ] Automatically create packages for Debian and RedHat based distributions using GitHub Actions
- [ ] Possibly more

## License

MIT License

Copyright (c) 2026 Katelyn 'KatLongLegs' Haworth.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

[0]: https://novelbits.io/bluetooth-address-privacy-ble/

[1]: https://github.com/furiousMAC/continuity/blob/master/messages/nearby_info.md

[2]: https://rustup.rs/

[3]: http://example.com

[4]: https://espresense.com/devices/apple

[5]: https://github.com/DavidSt49/watch-unlock-linux