import { expect,it } from 'vitest';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import Calendar, { factorsOnDay,monthDays,temperatureBadge } from './Calendar';
import { popupPosition } from './CalendarPopup';
import type { Signal, Snapshot } from './data';
it('lays out months from Monday, including adjoining months',()=>{const days=monthDays('2026-10');expect(days).toHaveLength(35);expect(days[0]).toBe('2026-09-28');expect(days[34]).toBe('2026-11-01');});
it('shows every day of a holiday span and only the matching event day',()=>{const holiday={id:'h',category:'schools',source:'schools',title:'Half term',date:'2026-10-26',end_date:'2026-10-30',detail:'',venue:null} as Signal;const event={...holiday,id:'e',category:'events',title:'Concert',date:'2026-10-28',end_date:null} as Signal;expect(factorsOnDay([holiday,event],'2026-10-27','all','')).toEqual([holiday]);expect(factorsOnDay([holiday,event],'2026-10-28','events','concert')).toEqual([event]);expect(factorsOnDay([holiday,event],'2026-10-31','all','')).toEqual([]);});

it('colours forecast highs across temperature bands, including freezing and unavailable data',()=>{
 for(const [title,high,band] of [
  ['-12–-4°C · Snow',-4,'cold'],['0–5°C · Clear sky',5,'cold'],
  ['3–6°C · Rain',6,'cool'],['6–11°C · Cloudy',11,'cool'],
  ['8–12°C · Cloudy',12,'mild'],['14–19°C · Cloudy',19,'mild'],
  ['14–20°C · Clear sky',20,'warm'],['20–25°C · Clear sky',25,'warm'],
  ['20–26°C · Clear sky',26,'hot'],['24–34°C · Clear sky',34,'hot']
 ] as const){expect(temperatureBadge(title)).toMatchObject({high,band});}
 expect(temperatureBadge('-12–-4°C · Snow')?.description).toContain('-12–-4°C · Snow');
 expect(temperatureBadge('4–8°C · Light rain')?.condition).toBe('Light rain');
 expect(temperatureBadge('7–7°C · ')?.condition).toBe('Conditions unavailable');
 expect(temperatureBadge('Weather unavailable')).toBeNull();
 expect(temperatureBadge('')).toBeNull();
});

it('uses only the calendar weeks needed, including leap years',()=>{
 expect(monthDays('2027-02')).toHaveLength(28);
 expect(monthDays('2026-08')).toHaveLength(42);
 expect(monthDays('2028-02')).toContain('2028-02-29');
});

it('provides individual badge buttons and overflow without rendering details below the calendar',()=>{
 const signals=Array.from({length:5},(_,n):Signal=>({id:`film-${n}`,category:'cinema',source:'cinema',title:`Example film ${n+1}`,date:'2026-10-19',end_date:null,time:null,venue:null,distance_miles:null,url:null,detail:''}));
 const snapshot:Snapshot={location:{postcode:'SW1A 1AA',latitude:51.5,longitude:-.1,country:'England',council:'Westminster'},radius_miles:5,lookahead_days:30,from:'2026-10-19',through:'2026-11-17',sources:[],signals};
 const rendered:string[]=[];
 const html=renderToStaticMarkup(createElement(Calendar,{snapshot,filter:'all',search:'',renderSignal:s=>{rendered.push(s.id);return null;}}));
 expect(html).toContain('Cinema release: Example film 1');
 expect(html).toContain('Cinema release: Example film 3');
 expect(html).not.toContain('Cinema release: Example film 4');
 expect(html).toContain('+2 more');
 expect(html).toMatch(/<button[^>]*aria-label="Cinema release: Example film 1"[^>]*aria-haspopup="dialog"/);
 expect(html).not.toContain('calendar-selected');
 expect(rendered).toHaveLength(0);
});

it('keeps anchored popups inside the viewport, including corners and compact screens',()=>{
 const size={width:360,height:320};
 expect(popupPosition({left:100,right:200,top:200,bottom:230},size,{width:1200,height:800}).side).toBe('right');
 expect(popupPosition({left:950,right:1050,top:600,bottom:630},size,{width:1200,height:800}).side).toBe('left');
 for(const viewport of [{width:1200,height:800},{width:360,height:600}]){
  for(const anchor of [{left:10,right:90,top:10,bottom:40},{left:viewport.width-100,right:viewport.width-10,top:viewport.height-50,bottom:viewport.height-20}]){
   const position=popupPosition(anchor,size,viewport);
   expect(position.left).toBeGreaterThanOrEqual(12);
   expect(position.top).toBeGreaterThanOrEqual(12);
   expect(position.left+Math.min(size.width,viewport.width-24)).toBeLessThanOrEqual(viewport.width-12);
   expect(position.top+size.height).toBeLessThanOrEqual(viewport.height-12);
  }
 }
});
