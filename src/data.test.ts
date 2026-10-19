import { describe,expect,it } from 'vitest';
import { groupSignals,needsRefresh,type Signal,type Snapshot } from './data';
const signal:Signal={id:'a',category:'schools',source:'schools',title:'Half term',date:'2026-10-19',end_date:'2026-10-30',time:null,venue:null,distance_miles:null,url:null,detail:'Council dates'};
describe('loaded factors',()=>{
 it('keeps an ongoing holiday in the current view and filters locally',()=>{
  const groups=groupSignals([signal],'2026-10-21','holidays','half');expect(groups[0][0]).toBe('2026-10-21');expect(groups[0][1]).toEqual([signal]);
  expect(groupSignals([signal],'2026-10-21','events','')).toEqual([]);
 });
 it('requests an update at 24 hours without treating disabled sources as stale',()=>{
  const now=Date.parse('2026-10-20T10:00:00Z');const source={id:'schools',state:'fresh',note:'',count:0,updated_at:'2026-10-19T10:00:00Z'};
  expect(needsRefresh({sources:[source]} as Snapshot,now)).toBe(true);
  expect(needsRefresh({sources:[{...source,updated_at:'2026-10-19T10:00:01Z'}]} as Snapshot,now)).toBe(false);
  expect(needsRefresh({sources:[{...source,state:'disabled',updated_at:null}]} as Snapshot,now)).toBe(false);
 });
});
