import { useEffect, useRef, useState } from 'react';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ArrowUpRight, Download, LoaderCircle, RefreshCw } from 'lucide-react';
import { desktop, openSource } from './api';
import { version } from '../package.json';

export default function SoftwareUpdates({ blocked, onBusyChange }:{blocked:boolean;onBusyChange:(busy:boolean)=>void}) {
 const [state,setState]=useState<'idle'|'checking'|'available'|'downloading'|'ready'|'restarting'|'current'|'error'>('idle');
 const [release,setRelease]=useState<{version:string;notes?:string}|null>(null),[error,setError]=useState('');
 const [download,setDownload]=useState({bytes:0,total:0});
 const update=useRef<Update|null>(null),working=useRef(false);
 useEffect(()=>()=>{void update.current?.close().catch(()=>{});},[]);
 const workingNow=state==='checking'||state==='downloading'||state==='restarting';
 async function checkNow(){
  if(blocked||working.current)return;
  working.current=true;onBusyChange(true);setState('checking');setError('');setRelease(null);
  try {
   await update.current?.close();update.current=null;
   const found=await check({timeout:20000});update.current=found;
   if(found){setRelease({version:found.version,notes:found.body});setState('available');}
   else setState('current');
  }catch {setError('Couldn’t check for updates. Check your connection and try again.');setState('error');}
  finally{working.current=false;onBusyChange(false);}
 }
 async function install(){
  if(blocked||working.current||!update.current)return;
  working.current=true;onBusyChange(true);setState('downloading');setError('');setDownload({bytes:0,total:0});
  try {
   await update.current.downloadAndInstall(event=>{
    if(event.event==='Started')setDownload({bytes:0,total:event.data.contentLength??0});
    if(event.event==='Progress')setDownload(d=>({...d,bytes:d.bytes+event.data.chunkLength}));
   },{timeout:120000});
   setState('ready');
  }catch {setError('The update could not be installed. Downloads must pass a signature check. Check for updates to try again.');setState('error');}
  finally{working.current=false;onBusyChange(false);}
 }
 async function restart(){
  if(blocked||working.current)return;
  working.current=true;onBusyChange(true);setState('restarting');setError('');
  try{await relaunch();}
  catch{setError('Couldn’t restart automatically. Quit and reopen Local Outlook to finish the update.');setState('ready');working.current=false;onBusyChange(false);}
 }
 const percent=download.total?Math.min(100,Math.round(download.bytes/download.total*100)):undefined;
 return <section className="settings-section software-updates" aria-labelledby="software-updates-title">
  <div className="settings-section-title"><Download size={21}/><div><h2 id="software-updates-title">App updates</h2><p>Local Outlook {version} · Checks happen only when you ask.</p></div></div>
  <div className="app-update-actions">
   {state!=='ready'&&state!=='restarting'&&<button type="button" className="button secondary small" disabled={!desktop||blocked||workingNow} onClick={()=>void checkNow()}>{state==='checking'?<LoaderCircle size={16} className="spin"/>:<RefreshCw size={16}/>} {state==='checking'?'Checking…':'Check for updates'}</button>}
   <a href="https://github.com/Jamie9192/local-outlook/releases" target="_blank" rel="noreferrer" onClick={e=>{e.preventDefault();void openSource(e.currentTarget.href);}}>Release history <ArrowUpRight size={14}/></a>
  </div>
  {!desktop&&<p className="app-update-status">Open the desktop app to check for updates.</p>}
  {blocked&&!workingNow&&<p className="app-update-status">Finish the current action and save your settings before updating.</p>}
  <div className="app-update-status" role="status" aria-live="polite">
   {state==='current'&&<p>You’re using the latest available version.</p>}
   {state==='available'&&release&&<><strong>Version {release.version} is available.</strong>{release.notes&&<details className="app-update-notes"><summary>What’s new</summary><p>{release.notes}</p></details>}<p>Download and install when you’re ready. The app may close during installation; your settings and cache are kept.</p><button type="button" className="button primary small" disabled={blocked} onClick={()=>void install()}><Download size={16}/>Download &amp; install</button></>}
   {state==='downloading'&&<><p>{download.bytes?`Downloaded ${(download.bytes/1048576).toFixed(1)} MB${percent!==undefined?` · ${percent}%`:''}`:'Downloading and verifying the update…'}</p><progress aria-label="Update download" max={100} value={percent}/></>}
   {(state==='ready'||state==='restarting')&&<><strong>Update installed. Restart to use the new version.</strong><button type="button" className="button primary small" disabled={blocked||workingNow} onClick={()=>void restart()}><RefreshCw size={16}/>{state==='restarting'?'Restarting…':'Restart app'}</button></>}
  </div>
  {error&&<p className="app-update-error" role="alert">{error}</p>}
 </section>;
}
