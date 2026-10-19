# Local Outlook

A Mac and Windows desktop app that brings upcoming local events, UK cinema
releases, council school holidays, bank holidays and weather into a daily agenda and calendar.
Enter a UK postcode after setting up your sources. The initial view covers the
next 30 days within five miles; both are configurable.

This version shows factors to inform sales planning. It does not calculate sales
forecasts, predict attendance or assign invented percentage uplifts.

## Install a family preview

Download an installer from [GitHub Releases](https://github.com/Jamie9192/local-outlook/releases).
The universal Mac DMG supports Apple Silicon and Intel on macOS 11 or newer:
open it and drag Local Outlook into Applications. Windows users can run the
setup EXE; it installs WebView2 if needed. Rust and Node.js are not needed.

These early builds are not Apple-notarised or Windows publisher-signed. Your
operating system may warn or block the first launch. They are intended for
trusted family testing. Configure your own source credentials in Settings.

Settings → App updates lets you check GitHub, read the release notes, and choose
Download & install. Restart when prompted. Checks are manual; nothing is
downloaded or installed automatically. Update packages require a valid signature.
Settings, cached factors and provider credentials stay on the device.

The code is [MIT licensed](LICENSE). Dependencies and data providers retain their
own licences and terms; [third-party notices](THIRD_PARTY_NOTICES.txt) are included
in the application bundle.

## Run the app

Development requires Node.js 22.12+ (24 LTS recommended), Rust 1.90+ and the
platform's desktop build tools. Application dependencies are locked in
`package-lock.json` and `src-tauri/Cargo.lock`.

On macOS:

```sh
sh scripts/setup-macos.sh
npm run desktop -- dev
```

On Windows, run PowerShell (the build tool installer may request elevation):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup-windows.ps1
npm run desktop -- dev
```

Packaged users do not need Rust or Node. The Windows installer installs the
WebView2 runtime if necessary. `npm run dev` opens a browser interface preview;
API calls and credential storage are available in the desktop app.

## Connect data sources

Use Settings to enable sources, enter credentials, test them and save. Saving
never makes a provider request. Test connection does, and does not save your key.

| Source | Setup and business requirements |
| --- | --- |
| [Postcodes.io](https://postcodes.io/docs/licences/) | No API key for postcode lookup. Great Britain data is open; commercial use of Northern Ireland postcode data requires the appropriate NI Land & Property Services licence. BT searches remain disabled until you confirm that licence in Settings. |
| [Ticketmaster](https://developer.ticketmaster.com/products-and-docs/apis/discovery-api/v2/) | Developer API key; check [business-use terms](https://developer.ticketmaster.com/support/terms-of-use/). Venue coordinates are independently checked against the configured straight-line radius from the postcode centre. |
| [Skiddle](https://www.skiddle.com/api/) | Optional API key, disabled initially. Written commercial approval is required. |
| [Wikidata](https://www.wikidata.org/wiki/Wikidata:Licensing) | Default cinema source: free CC0 data, no key. Only exact UK-tagged film dates are included. Coverage is limited, and missing release-format information is disclosed; dates are not automatically confirmed theatrical releases. |
| [TMDB](https://developer.themoviedb.org/docs/faq) | Optional cinema source in Settings. API **Read Access Token**, not the shorter v3 API key. Business use requires a commercial licence. UK theatrical dates are checked separately from worldwide and streaming dates. Releases do not establish local cinema screenings. |
| [Housemait](https://housemait.com/school-term-dates/data) | No key; CC BY 4.0 attribution. Council dates can be incomplete, and academies, independent schools and INSET days may differ. The council's original source is linked. |
| [MET Norway](https://api.met.no/doc/TermsOfService) | No API key. Enter a real contact email or HTTPS website so requests can be identified. CC BY 4.0 attribution is required. Weather covers approximately nine days, not the entire 30-day planning period. |
| [GOV.UK](https://www.api.gov.uk/gds/bank-holidays/) | No key; public sector information under the Open Government Licence v3.0. Nation-specific bank holiday calendars. |

A connection test confirms technical access, not commercial permission. No source
can guarantee all local events. Missing venue coordinates, API result limits and
incomplete council coverage are reported in source status details.

## Refresh and offline behaviour

- Check area makes on-demand requests for missing or outdated source data.
- Fresh cached results are reused. Filters and searching loaded factors are local.
- Refresh checks enabled sources even when the normal cache is fresh, while
  respecting provider `Expires` headers and limits.
- Opening the app makes no API calls. If a view needs updating, a prompt offers
  Refresh now or Later. Returning to the app checks the age locally.
- Each source has its own successful timestamp. A failed refresh retains its old
  results and timestamp with an error, instead of reporting the source as fresh.
- When the calendar day changes, overlapping cached results remain visible;
  missing coverage triggers the refresh prompt.
- SQLite holds settings and cached data in the operating system's app-data folder
  for `uk.localoutlook.desktop`. Cached responses are pruned after 30 days.
- API keys are stored only in macOS Keychain or Windows Credential Manager.
  There is no plain-text key fallback. Clearing cached results preserves keys
  and settings; removing a credential is a separate Settings action.

The calendar shares cached data with the agenda. Select a date for details, use
arrow keys to move between dates, and switch months without network requests.
Holiday spans are shown on every day they cover.

School holidays are normalised from council term boundaries and explicit holiday
spans. Derived breaks include weekends and are labelled accordingly. No holiday
is inferred outside the available calendar. Event cancellations and date changes
are shown in expanded event details.

## Checks and packaging

```sh
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

An explicit opt-in test calls the public postcode, school, bank holiday and
Wikidata APIs, then verifies offline cache reuse and failed-refresh retention:

```sh
cargo test --manifest-path src-tauri/Cargo.toml live_public_sources_and_offline_cache -- --ignored
```

Build on the corresponding operating system:

```sh
# Mac: Apple Silicon + Intel
npm run desktop -- build --target universal-apple-darwin --bundles app,dmg
# Windows
npm run desktop -- build --bundles nsis
```

The desktop workflow tests and packages on native Mac and Windows runners.
Version tags trigger the release workflow, which uploads signed updater packages
to a draft and publishes only after both platform builds and the update manifest
pass. See [release maintenance](docs/releasing.md). Family previews use local/ad-hoc
Mac signing; Apple notarisation and Windows publisher signing are separate setup.

## Credits

Postcode data: Postcodes.io, OS OpenData and associated Royal Mail/ONS/NRS data
attributions described in the provider's licence. School dates: Housemait
(CC BY 4.0), compiled from the linked council calendars. Weather: MET Norway
(CC BY 4.0); forecast values are aggregated into available daily intervals.
Bank holiday data: GOV.UK, Open Government Licence v3.0.

TMDB logo: official asset from the provider’s developer documentation
(https://files.readme.io/29c6fee-blue_short.svg).
This product uses the TMDB API but is not endorsed or certified by TMDB. TMDB,
Ticketmaster and Skiddle remain independent data providers. Review provider
requirements before enabling them for commercial operation.

The cinema-source investigation and source-specific limitations are recorded in
[docs/cinema-sources.md](docs/cinema-sources.md).
