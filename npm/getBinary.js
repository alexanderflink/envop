const { Binary } = require('binary-install');
const os = require('os');

function getPlatform() {
    const type = os.type();
    const arch = os.arch();

    if (type === 'Windows_NT' && arch === 'x64') return 'x86_64-windows';
    if (type === 'Linux' && arch === 'x64') return 'x86_64-linux';
    if (type === 'Darwin' && arch === 'x64') return 'x86_64-macos';
    if (type === 'Darwin' && arch === 'arm64') return 'aarch64-macos';

    throw new Error(`Unsupported platform: ${type} ${arch}`);
}

function getBinary() {
    const platform = getPlatform();
    const version = require('../package.json').version;
    const url = `https://github.com/alexanderflink/envop/releases/download/v${ version }/envop-${ platform }.tar.gz`;
    const name = 'envop';
    return new Binary(name, url);
}

module.exports = getBinary;