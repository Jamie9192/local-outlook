export type ProviderId = 'ticketmaster'|'skiddle'|'cinema'|'schools'|'weather'|'bank_holidays';
export type Category = 'events'|'cinema'|'schools'|'weather'|'bank_holidays';
export interface Settings { postcode:string; radius_miles:number; lookahead_days:number; enabled:ProviderId[]; weather_contact:string; cinema_source:'wikidata'|'tmdb'; ni_licence_confirmed:boolean }
export interface Signal { id:string; category:Category; source:ProviderId; title:string; date:string; end_date:string|null; time:string|null; venue:string|null; distance_miles:number|null; url:string|null; detail:string }
export interface SourceStatus { id:ProviderId; state:'disabled'|'needs_setup'|'not_loaded'|'fresh'|'partial'|'stale'|'error'; note:string; updated_at:string|null; count:number }
export interface Snapshot { location:{postcode:string;latitude:number;longitude:number;country:string;council:string};radius_miles:number;lookahead_days:number;from:string;through:string;signals:Signal[];sources:SourceStatus[] }
export interface Bootstrap { settings:Settings;credentials:ProviderId[];snapshot:Snapshot|null }
export const defaultSettings:Settings={postcode:'',radius_miles:5,lookahead_days:30,enabled:['ticketmaster','cinema','schools','weather','bank_holidays'],weather_contact:'',cinema_source:'wikidata',ni_licence_confirmed:false};
export const providers:{id:ProviderId;name:string;description:string;licence:string;url:string;credential?:string}[]=[
 {id:'ticketmaster',name:'Ticketmaster',description:'Concerts, sport, theatre and more, filtered to your radius.',licence:'Free developer key. Confirm your business use is permitted under Ticketmaster’s terms.',url:'https://developer.ticketmaster.com/support/terms-of-use/',credential:'API key'},
 {id:'skiddle',name:'Skiddle',description:'An optional second source for nearby events.',licence:'Written approval from Skiddle is required for commercial use.',url:'https://www.skiddle.com/api/',credential:'API key'},
 {id:'cinema',name:'Cinema releases',description:'UK film dates from open data, with an optional licensed TMDB source.',licence:'Wikidata is free (CC0), with limited UK coverage. TMDB is optional and requires a commercial licence.',url:'https://developer.themoviedb.org/docs/faq',credential:'API Read Access Token'},
 {id:'schools',name:'School holidays',description:'Council calendars matched to your postcode, via Housemait.',licence:'Free for commercial use with attribution (CC BY 4.0). Schools may set their own dates.',url:'https://housemait.com/school-term-dates/data'},
 {id:'weather',name:'Weather',description:'Temperature, conditions and precipitation from MET Norway.',licence:'Open data (CC BY 4.0). An identifying contact email or website is required. Forecasts cover approximately nine days.',url:'https://api.met.no/doc/TermsOfService'},
 {id:'bank_holidays',name:'Bank holidays',description:'Official dates for England and Wales, Scotland or Northern Ireland.',licence:'Free GOV.UK data under the Open Government Licence.',url:'https://www.gov.uk/bank-holidays'}
];
export const categoryNames:Record<Category,string>={events:'Local event',cinema:'Cinema release',schools:'School holiday',weather:'Weather',bank_holidays:'Bank holiday'};
export const statusNames:Record<SourceStatus['state'],string>={disabled:'Off',needs_setup:'Set up',not_loaded:'Not checked',fresh:'Up to date',partial:'Limited coverage',stale:'Update due',error:'Needs attention'};
export function dayLabel(day:string,options:Intl.DateTimeFormatOptions={day:'numeric',month:'short'}) {return new Intl.DateTimeFormat('en-GB',options).format(new Date(day+'T12:00:00Z'));}
export function ageLabel(time:string|null) {
 if(!time)return 'Not checked yet';const delta=Math.max(0,Date.now()-new Date(time).getTime());
 if(delta<60_000)return 'Just now';if(delta<3_600_000)return `${Math.floor(delta/60_000)} min ago`;
 if(delta<86_400_000)return `${Math.floor(delta/3_600_000)} hr ago`;return `${Math.floor(delta/86_400_000)} days ago`;
}
export function needsRefresh(s:Snapshot,now=Date.now()) {return s.sources.some(p=>p.state!=='disabled'&&p.state!=='needs_setup'&&(p.state==='not_loaded'||p.state==='stale'||!p.updated_at||now-Date.parse(p.updated_at)>=86_400_000));}
export function groupSignals(signals:Signal[],from:string,filter:string,search:string) {
 const groups=new Map<string,Signal[]>();
 for(const signal of signals) {
  if(filter!=='all' && (filter==='holidays'?!['schools','bank_holidays'].includes(signal.category):signal.category!==filter))continue;
  if(search && ![signal.title,signal.venue,signal.detail].some(t=>t?.toLowerCase().includes(search.toLowerCase())))continue;
  const day=signal.date<from?from:signal.date;groups.set(day,[...(groups.get(day)??[]),signal]);
 }
 return [...groups].sort(([a],[b])=>a.localeCompare(b));
}
