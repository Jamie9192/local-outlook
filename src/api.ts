import { invoke, isTauri } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { defaultSettings, type Bootstrap, type ProviderId, type Settings, type Snapshot } from './data';
export const desktop=isTauri();
export async function boot():Promise<Bootstrap> {return desktop?invoke('bootstrap'):{settings:defaultSettings,credentials:[],snapshot:null};}
function requireDesktop(){if(!desktop)throw new Error('Open the desktop app to connect sources and save settings. This browser view is a preview.');}
export async function checkArea(postcode:string,force:boolean):Promise<Snapshot>{requireDesktop();return invoke('check_area',{postcode,force});}
export async function saveSettings(settings:Settings,secrets:Partial<Record<ProviderId,string>>,remove:ProviderId[]):Promise<Bootstrap>{requireDesktop();return invoke('save_settings',{request:{settings,secrets,remove_secrets:remove}});}
export async function clearCache(){requireDesktop();return invoke('clear_cache');}
export async function testProvider(id:ProviderId,key?:string,contact?:string,cinemaSource?:string):Promise<string>{requireDesktop();return invoke('test_provider',{id,key:key??null,contact:contact??null,cinemaSource:cinemaSource??null});}
export async function openSource(url:string){const parsed=new URL(url);if(parsed.protocol!=='https:')return;if(desktop)await openUrl(url);else window.open(url,'_blank','noopener,noreferrer');}
