import { readFileSync, readdirSync, existsSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { spawnSync } from 'node:child_process';

// Include dependency licence texts in the packaged app as well as in the source.
const cargo=spawnSync('cargo',['metadata','--format-version','1','--locked','--manifest-path','src-tauri/Cargo.toml'],{encoding:'utf8',maxBuffer:32*1024*1024});
if(cargo.status!==0)throw new Error(cargo.stderr||'Unable to read locked Rust dependencies');
const sections=['Local Outlook — third-party notices\n\nLocal Outlook is MIT licensed. Dependency licences and provider terms retain their own conditions. This file includes build and platform dependencies as well as application dependencies.\n\nThe TMDB logo is an official provider asset, used for attribution when that optional provider is selected. This product uses the TMDB API but is not endorsed or certified by TMDB. Data provider terms are linked in README.md and in Settings.'];
const packages=[];
for(const p of JSON.parse(cargo.stdout).packages){
 if(p.name==='local-outlook')continue;
 packages.push({name:`Rust: ${p.name} ${p.version}`,dir:dirname(p.manifest_path),license:p.license,licenseFile:p.license_file,repository:p.repository,authors:p.authors,source:`https://crates.io/crates/${p.name}/${p.version}`});
}
for(const [path,p] of Object.entries(JSON.parse(readFileSync('package-lock.json','utf8')).packages)){
 if(!path||p.dev)continue;
 const info=JSON.parse(readFileSync(join(path,'package.json'),'utf8'));
 packages.push({name:`JavaScript: ${info.name} ${info.version}`,dir:path,license:p.license,repository:typeof info.repository==='string'?info.repository:info.repository?.url});
}
const missing=[];
for(const p of packages.sort((a,b)=>a.name.localeCompare(b.name))){
 const names=readdirSync(p.dir).filter(n=>/^(licen[sc]e|copying|notice|ofl)([._-]|$)/i.test(n));
 if(p.licenseFile&&existsSync(join(p.dir,p.licenseFile))&&!names.includes(p.licenseFile))names.push(p.licenseFile);
 const texts=names.filter(n=>{try{return readFileSync(join(p.dir,n)).length<200000;}catch{return false;}}).map(n=>`${n}\n${readFileSync(join(p.dir,n),'utf8')}`);
 if(!texts.length){
  // Some registry archives omit standalone licence files. Preserve their declared
  // authors and source location alongside the full text of a permitted licence.
  const id=/\bMIT\b/.test(p.license)?'MIT':p.license;
  const fallback=join('licenses',`${id}.txt`);
  if(existsSync(fallback))texts.push(`Standard ${id} licence text (SPDX license-list-data)\n${readFileSync(fallback,'utf8')}`);
  else missing.push(p.name);
 }
 sections.push(`${'='.repeat(72)}\n${p.name}\nLicence: ${p.license||'See package metadata'}\n${p.repository||''}\n${p.source||''}\n${p.authors?.length?`Upstream authors: ${p.authors.join('; ')}`:''}\n\n${texts.join('\n\n')||'See upstream repository for the licence text.'}`);
}
writeFileSync('THIRD_PARTY_NOTICES.txt',sections.join('\n\n')+'\n');
console.log(`Included licence notices for ${packages.length} dependencies.`);
if(missing.length){console.error('Packages without packaged licence texts:\n'+missing.join('\n'));process.exitCode=1;}
