import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join, delimiter } from 'node:path';
import { homedir } from 'node:os';
const require = createRequire(import.meta.url);
const cli = join(dirname(require.resolve('@tauri-apps/cli/package.json')), 'tauri.js');
const child = spawn(process.execPath, [cli, ...process.argv.slice(2)], {
  stdio: 'inherit', env: { ...process.env, PATH: join(homedir(), '.cargo', 'bin') + delimiter + process.env.PATH }
});
child.on('exit', code => process.exit(code ?? 1));
child.on('error', error => { console.error(error.message); process.exit(1); });
