# Cinema release sources

Checked on 19 October 2026. The app defaults to free Wikidata data. TMDB remains
an optional commercially licensed source in Settings.

[Wikidata structured data](https://www.wikidata.org/wiki/Wikidata:Licensing) is CC0,
with a [public query service](https://www.wikidata.org/wiki/Wikidata:Data_access).
A live query for the next 30 days returned two UK-tagged dates, demonstrating
that this is a usable free source but has limited coverage. The app requires a
UK place qualifier and an exact day, ignores deprecated statements, excludes
explicitly non-theatrical formats, and discloses when release format is absent.
It never presents an unqualified worldwide date as a UK cinema date.

The [FDA industry calendar](https://filmdistributorsassociation.com/release-schedule/all-future-releases/)
is more comprehensive, is updated weekly and says its data comes from Comscore.
No explicit open-data reuse licence was established during this investigation.
The app links to it for checking releases; it does not bulk-copy its schedule.

[Clusterflick](https://clusterflick.com/data-licence/) permits commercial reuse of
its own screening records under CC BY 4.0, but explicitly excludes TMDB film
metadata, including release dates. A first local screening is also different
from a national release date, so the app does not substitute one for the other.

[TMDB](https://developer.themoviedb.org/docs/faq) provides useful country-specific
theatrical dates, but requires a commercial licence for business use. It is
available as an opt-in choice; it is not required for the free cinema source.

All sources can change dates. Each result links to its evidence. Community
coverage is always marked limited, including when no dates are returned.
