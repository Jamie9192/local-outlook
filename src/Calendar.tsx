import { useCallback, useEffect, useState, type ReactNode } from 'react';
import { ChevronLeft, ChevronRight, CalendarDays, CloudSun, CloudRain, CloudSnow, CloudFog, CloudOff, Cloud, Sun, Film, Music2, GraduationCap } from 'lucide-react';
import { categoryNames, dayLabel, type Signal, type Snapshot } from './data';
import CalendarPopup from './CalendarPopup';

const categoryIcons={events:Music2,cinema:Film,schools:GraduationCap,bank_holidays:CalendarDays,weather:CloudSun};
function weatherIcon(condition:string){return /snow|sleet/i.test(condition)?CloudSnow:/rain|showers/i.test(condition)?CloudRain:/fog/i.test(condition)?CloudFog:condition==='Clear sky'?Sun:/clear|partly/i.test(condition)?CloudSun:Cloud;}

export function monthDays(month:string) {
 const first=new Date(month+'-01T12:00:00Z');
 const offset=(first.getUTCDay()+6)%7;
 const start=new Date(first);start.setUTCDate(1-offset);
 const last=new Date(first);last.setUTCMonth(last.getUTCMonth()+1,0);
 const count=Math.ceil((offset+last.getUTCDate())/7)*7;
 return Array.from({length:count},(_,index)=>{const day=new Date(start);day.setUTCDate(start.getUTCDate()+index);return day.toISOString().slice(0,10);});
}
export function factorsOnDay(signals:Signal[],day:string,filter:string,search:string) {
 return signals.filter(s=>s.date<=day&&(s.end_date??s.date)>=day)
  .filter(s=>filter==='all'||(filter==='holidays'?['schools','bank_holidays'].includes(s.category):s.category===filter))
  .filter(s=>!search||[s.title,s.venue,s.detail].some(v=>v?.toLowerCase().includes(search.toLowerCase())));
}
export function temperatureBadge(title:string) {
 const match=/^(-?\d+)–(-?\d+)°C(?: · (.*))?$/.exec(title);
 if(!match)return null;
 const high=Number(match[2]);
 const band=high<6?'cold':high<12?'cool':high<20?'mild':high<26?'warm':'hot';
 return {high,band,condition:match[3]?.trim()||'Conditions unavailable',description:`${band[0].toUpperCase()+band.slice(1)} · Forecast high ${high}°C · ${title}`};
}
export default function Calendar({snapshot,filter,search,renderSignal}:{snapshot:Snapshot;filter:string;search:string;renderSignal:(s:Signal)=>ReactNode}) {
 const [month,setMonth]=useState(snapshot.from.slice(0,7)),[selected,setSelected]=useState(snapshot.from),[focused,setFocused]=useState(snapshot.from);
 useEffect(()=>{setMonth(snapshot.from.slice(0,7));setSelected(snapshot.from);setFocused(snapshot.from);},[snapshot.location.postcode,snapshot.from,snapshot.lookahead_days]);
 const [popup,setPopup]=useState<{signal:Signal|null;day:string;anchor:HTMLElement}|null>(null);
 const closePopup=useCallback((restoreFocus:boolean)=>{setPopup(null);if(restoreFocus&&popup?.anchor.isConnected)popup.anchor.focus({preventScroll:true});},[popup?.anchor]);
 useEffect(()=>{setPopup(null);},[month,snapshot,filter,search]);
 function openPopup(signal:Signal|null,day:string,anchor:HTMLElement){
  setSelected(day);setFocused(day);
  if(popup?.anchor===anchor&&popup.signal?.id===signal?.id){closePopup(true);return;}
  setPopup({signal,day,anchor});
 }
 const days=monthDays(month);
 const popupSignals=popup?factorsOnDay(snapshot.signals,popup.day,filter,search).sort((a,b)=>Number(b.category==='weather')-Number(a.category==='weather')):[];
 const today=new Intl.DateTimeFormat('en-CA',{timeZone:'Europe/London',year:'numeric',month:'2-digit',day:'2-digit'}).format(new Date());
 const weatherThrough=snapshot.signals.filter(s=>s.category==='weather').map(s=>s.date).sort().at(-1);
 const weatherUnavailable=popup&&!popup.signal&&(filter==='all'||filter==='weather')&&weatherThrough&&popup.day>weatherThrough;
 const weatherOff=snapshot.sources.find(source=>source.id==='weather')?.state==='disabled';
 const inRange=(day:string)=>day>=snapshot.from&&day<=snapshot.through;
 function moveMonth(delta:number){const d=new Date(month+'-01T12:00:00Z');d.setUTCMonth(d.getUTCMonth()+delta);const next=d.toISOString().slice(0,7);const anchor=next===snapshot.from.slice(0,7)?snapshot.from:next+'-01';setMonth(next);setSelected(anchor);setFocused(anchor);}
 function keyMove(day:string,key:string){const delta:{[key:string]:number}={ArrowLeft:-1,ArrowRight:1,ArrowUp:-7,ArrowDown:7};if(!(key in delta))return false;const d=new Date(day+'T12:00:00Z');d.setUTCDate(d.getUTCDate()+delta[key]);const next=d.toISOString().slice(0,10);if(!inRange(next))return true;setMonth(next.slice(0,7));setFocused(next);setTimeout(()=>document.querySelector<HTMLButtonElement>(`[data-calendar-date="${next}"]`)?.focus(),0);return true;}
 return <section className="calendar-view" aria-label="Calendar of loaded factors">
  <div className="calendar-heading"><h3>{dayLabel(month+'-01',{month:'long',year:'numeric'})}</h3><div><button className="icon-button" aria-label="Previous month" disabled={month<=snapshot.from.slice(0,7)} onClick={()=>moveMonth(-1)}><ChevronLeft size={19}/></button><button className="icon-button" aria-label="Next month" disabled={month>=snapshot.through.slice(0,7)} onClick={()=>moveMonth(1)}><ChevronRight size={19}/></button></div></div>
  <div className="calendar-scroll" role="region" aria-label="Calendar dates; scroll horizontally if needed" tabIndex={0}>
  <table className="calendar-table">
   <caption className="sr-only">Loaded factors by date. Temperature badges show forecast highs. Select an entry or a day to open its details in a popup. Arrow keys move between available dates.</caption>
   <thead><tr>{['Mon','Tue','Wed','Thu','Fri','Sat','Sun'].map(day=><th scope="col" key={day}>{day}</th>)}</tr></thead>
   <tbody>{Array.from({length:days.length/7},(_,week)=><tr key={week} className={days.slice(week*7,week*7+7).some(inRange)?undefined:'calendar-week-outside'}>{days.slice(week*7,week*7+7).map(day=>{
    const signals=factorsOnDay(snapshot.signals,day,filter,search);
    const entries=signals.filter(s=>s.category!=='weather');
    const weather=snapshot.signals.find(s=>s.category==='weather'&&s.date===day);
    const temperature=weather?temperatureBadge(weather.title):null;
    const condition=temperature?.condition??'Conditions unavailable',WeatherIcon=weatherIcon(condition);
    const dayPopup=popup?.day===day&&!popup.signal;
    return <td key={day} className={day.slice(0,7)!==month?'other-month':''}>
     <div className={`calendar-day ${selected===day?'chosen':''} ${day===today?'today':''} ${!inRange(day)?'unavailable':''}`}>
      <button type="button" className="calendar-day-heading calendar-date-select" disabled={!inRange(day)} tabIndex={focused===day?0:-1} data-calendar-date={day} aria-current={day===today?'date':undefined} aria-pressed={selected===day} aria-haspopup="dialog" aria-expanded={!!dayPopup} aria-label={`${dayLabel(day,{weekday:'long',day:'numeric',month:'long'})}, ${inRange(day)?`${signals.length} matching ${signals.length===1?'factor':'factors'}${entries.length?`, ${entries.slice(0,3).map(s=>s.title).join('; ')}`:''}${temperature?`, ${temperature.description}`:''}`:'outside loaded range'}`} onClick={e=>openPopup(null,day,e.currentTarget)} onKeyDown={e=>{if(keyMove(day,e.key))e.preventDefault();}}>
       <span className="calendar-number">{Number(day.slice(-2))}</span>
       {inRange(day)&&temperature&&<span className={`calendar-temperature ${temperature.band}`} title={temperature.description} aria-hidden="true"><span>{temperature.high}°</span><span className="temperature-unit">C</span></span>}
      </button>
      {inRange(day)&&<div className="calendar-day-factors">
       {weather?<button type="button" className="calendar-day-weather" title={weather.title} aria-label={`Weather: ${weather.title}`} aria-haspopup="dialog" aria-expanded={popup?.signal?.id===weather.id} onClick={e=>openPopup(weather,day,e.currentTarget)}><WeatherIcon size={14} aria-hidden="true"/><span>{condition}</span></button>:<span className="calendar-day-weather no-forecast" title={weatherThrough?`Forecast available through ${dayLabel(weatherThrough)}`:'No weather forecast is loaded'}><CloudOff size={14} aria-hidden="true"/><span>{weatherOff?'Weather off':'No forecast'}</span></span>}
       {entries.slice(0,3).map(s=>{const EntryIcon=categoryIcons[s.category];return <button type="button" className={`calendar-entry ${s.category}`} key={s.id} title={`${categoryNames[s.category]}: ${s.title}${s.time?` · ${s.time}`:''}`} aria-label={`${categoryNames[s.category]}: ${s.title}`} aria-haspopup="dialog" aria-expanded={popup?.signal?.id===s.id&&popup.day===day} onClick={e=>openPopup(s,day,e.currentTarget)}><EntryIcon size={13} aria-hidden="true"/><span>{s.title}</span></button>;})}
       {entries.length>3&&<button type="button" className="calendar-more" aria-label={`Show all ${entries.length} entries for ${dayLabel(day)}`} aria-haspopup="dialog" aria-expanded={!!dayPopup} onClick={e=>openPopup(null,day,e.currentTarget)}>+{entries.length-3} more</button>}
      </div>}
     </div>
    </td>;
   })}</tr>)}</tbody>
  </table>
  </div>
  <p className="calendar-note">Temperature badges show forecast highs. Calendar navigation uses cached information. Shaded dates are outside the loaded range; an empty date does not guarantee there are no local events.</p>
  {popup&&<CalendarPopup anchor={popup.anchor} title={popup.signal?.title??dayLabel(popup.day,{weekday:'long',day:'numeric',month:'long',year:'numeric'})} onClose={closePopup}>
   {popup.signal?renderSignal(popup.signal):<>
    {weatherUnavailable&&<p className="popup-coverage"><CloudOff size={17} aria-hidden="true"/><span>Weather is unavailable for this date. The loaded forecast runs through {dayLabel(weatherThrough)}.</span></p>}
    {popupSignals.length?<div className="calendar-popup-list">{popupSignals.map(s=>{const EntryIcon=categoryIcons[s.category];return <button type="button" key={s.id} className={`calendar-popup-list-entry ${s.category}`} onClick={()=>setPopup({...popup,signal:s})}><EntryIcon size={18} aria-hidden="true"/><span><strong>{s.title}</strong><small>{categoryNames[s.category]}{s.time?` · ${s.time}`:''}</small></span><ChevronRight size={16} aria-hidden="true"/></button>;})}</div>:<p className="calendar-popup-empty">No matching factors are loaded for this day. Check source coverage or change your filters.</p>}
   </>}
  </CalendarPopup>}
 </section>;
}
