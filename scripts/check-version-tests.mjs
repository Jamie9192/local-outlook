import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

test('release version checks accept Windows line endings and reject a mismatched tag',()=>{
 const dir=mkdtempSync(join(tmpdir(),'local-outlook-version-'));
 try{
  mkdirSync(join(dir,'src-tauri'));
  const json=JSON.stringify({version:'1.2.3',packages:{'':{version:'1.2.3'}}});
  for(const path of ['package.json','package-lock.json','src-tauri/tauri.conf.json'])writeFileSync(join(dir,path),json);
  const run=tag=>spawnSync(process.execPath,[resolve('scripts/check-version.mjs')],{cwd:dir,env:{...process.env,RELEASE_TAG:tag},encoding:'utf8'});
  for(const newline of ['\n','\r\n']){
   writeFileSync(join(dir,'src-tauri/Cargo.toml'),`[package]${newline}version = "1.2.3"${newline}`);
   writeFileSync(join(dir,'src-tauri/Cargo.lock'),`[[package]]${newline}name = "local-outlook"${newline}version = "1.2.3"${newline}`);
   const result=run('v1.2.3');assert.equal(result.status,0,result.stderr);
  }
  const wrong=run('v1.2.4');assert.notEqual(wrong.status,0);assert.match(wrong.stderr,/Tag must match/);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
