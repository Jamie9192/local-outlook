# Release maintenance

The repository publishes Mac universal DMGs, Windows NSIS installers and Tauri
updater packages through GitHub Releases. Ordinary development builds require no
signing secret. `src-tauri/tauri.release.conf.json` enables signed updater artifacts.

## Publish

1. Update the version in package.json, package-lock.json, Cargo.toml, Cargo.lock
   and tauri.conf.json. The interface reads the package version automatically.
2. Update docs/release-notes.md. Keep preview signing limitations explicit.
3. Run `npm run check:version`, `npm test`, `npm run build`, the Rust tests and Clippy.
4. Commit, push, tag the same commit as `vVERSION`, and push the tag.
5. The release workflow builds both platforms, uploads to a draft, checks that
   latest.json covers both Mac architectures and Windows, then publishes it.
   A failed platform build leaves the draft unpublished for inspection or retry.

Published releases are retained so existing update URLs and signatures keep
working. Old local installer copies can be moved to the Bin independently.
Do not replace the files of a published version; issue a higher version instead.

## Updater signing

Repository Actions secrets:

- `TAURI_SIGNING_PRIVATE_KEY`: encrypted Tauri updater private key content.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: its password.

The public verification key is in tauri.conf.json. Keep an encrypted backup of the
original private key and its password outside the repository. Losing or replacing
the key prevents installed copies from trusting future updates.

For a local signed-updater build, load those two variables into the process
environment and run:

```sh
npm run desktop -- build --target universal-apple-darwin --bundles app,dmg --config src-tauri/tauri.release.conf.json
```

The feed is `https://github.com/Jamie9192/local-outlook/releases/latest/download/latest.json`.
Each entry must use the matching file signature and a version-specific HTTPS URL.
Universal Mac packages cover both `darwin-aarch64` and `darwin-x86_64`.
Only publish a completed release as latest. In-app update checks never contain
the user's postcode, provider credentials or cached factors.

## Family preview and future publisher signing

Current previews use ad-hoc Mac signatures and have no Apple notarisation or
Windows publisher certificate. Updater signatures authenticate update packages;
they do not establish an Apple or Microsoft publisher identity. macOS or Windows
may warn or block the first launch on another computer.

For wider Mac distribution, use a Developer ID Application certificate, enable
the hardened runtime, and notarise/staple using Tauri's documented Apple signing
flow. Configure distribution credentials as protected repository secrets, never
commit them. For Windows, add an appropriate publisher-signing process.

## Licences

`npm run notices` creates THIRD_PARTY_NOTICES.txt from locked runtime JavaScript
packages and Rust package metadata, including other target/build dependencies.
It retains packaged licence texts and uses locally stored SPDX standard texts
where a crate archive omits its licence file, alongside upstream authors and
source links. A missing fallback fails the build for review. The bundle includes
the application MIT licence and these notices.
