import { readFileSync } from 'node:fs';
import { join, basename } from 'node:path';
import assert from 'node:assert/strict';

const release=JSON.parse(readFileSync(process.argv[2],'utf8'));
const version=release.tag_name.replace(/^v/,'');
assert.match(version,/^\d+\.\d+\.\d+$/);
const one=suffix=>{const found=release.assets.filter(a=>a.name.endsWith(suffix));assert.equal(found.length,1,`Expected one ${suffix} artifact`);return found[0];};
function entry(suffix){
 const asset=one(suffix),sig=release.assets.find(a=>a.name===`${asset.name}.sig`);
 assert.ok(sig,`Missing signature for ${asset.name}`);
 assert.equal(sig.name,basename(sig.name));
 const signature=readFileSync(join(process.argv[3],sig.name),'utf8').trim();
 assert.match(signature,/^[A-Za-z0-9+/]+={0,2}$/,'Invalid signature encoding');
 const url=new URL(asset.browser_download_url);
 assert.equal(url.protocol,'https:');assert.equal(url.hostname,'github.com');
 assert.ok(url.pathname.includes(`/releases/download/${release.tag_name}/`),'Use version-specific download URLs');
 return {url:url.href,signature};
}
const mac=entry('.app.tar.gz'),windows=entry('.exe');
const manifest={version,notes:release.body??'',pub_date:new Date().toISOString(),platforms:{'darwin-aarch64':mac,'darwin-x86_64':mac,'windows-x86_64':windows}};
process.stdout.write(JSON.stringify(manifest,null,2)+'\n');
