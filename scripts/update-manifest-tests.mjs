import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync,writeFileSync,rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join,resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

test('a completed update feed maps both Mac architectures and refuses partial or unversioned releases',()=>{
 const dir=mkdtempSync(join(tmpdir(),'local-outlook-feed-'));
 try{
  const names=['Local.Outlook.app.tar.gz','Local.Outlook.app.tar.gz.sig','Local.Outlook.setup.exe','Local.Outlook.setup.exe.sig'];
  const release={tag_name:'v1.2.3',body:'Release notes',assets:names.map(name=>({name,browser_download_url:`https://github.com/example/local-outlook/releases/download/v1.2.3/${name}`}))};
  for(const name of names.filter(n=>n.endsWith('.sig')))writeFileSync(join(dir,name),'c2lnbmF0dXJl');
  const path=join(dir,'release.json');
  const run=data=>{writeFileSync(path,JSON.stringify(data));return spawnSync(process.execPath,[resolve('scripts/create-update-manifest.mjs'),path,dir],{encoding:'utf8'});};
  const success=run(release);assert.equal(success.status,0,success.stderr);
  const manifest=JSON.parse(success.stdout);assert.equal(manifest.version,'1.2.3');assert.deepEqual(manifest.platforms['darwin-aarch64'],manifest.platforms['darwin-x86_64']);assert.ok(manifest.platforms['windows-x86_64'].signature);
  const draft=run({...release,assets:release.assets.map(a=>({...a,browser_download_url:a.browser_download_url.replace('/download/v1.2.3/','/download/untagged-abc123/')}))});assert.equal(draft.status,0,draft.stderr);assert.ok(Object.values(JSON.parse(draft.stdout).platforms).every(p=>p.url.includes('/download/v1.2.3/')));
  assert.notEqual(run({...release,assets:release.assets.filter(a=>!a.name.endsWith('.exe.sig'))}).status,0);
  assert.notEqual(run({...release,assets:release.assets.map(a=>({...a,browser_download_url:a.browser_download_url.replace('/download/v1.2.3/','/latest/download/')}))}).status,0);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
