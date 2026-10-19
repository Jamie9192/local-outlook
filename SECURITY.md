# Security

Report a suspected vulnerability through this repository's private security
advisory form. Do not include API keys, cached location data or other personal
information in public issues.

API credentials stay in macOS Keychain or Windows Credential Manager. Settings
and cached factors stay in the operating system's application-data directory.
They are never included in releases or uploaded by the updater.

Software update checks contact GitHub only when requested. Downloads are
verified against the updater public key embedded in the app before installation.
Updater signing is separate from operating system code signing. Early family
preview builds are not Apple-notarised or Windows publisher-signed; the operating
system may warn or block them on first launch.
