/* eslint-disable no-undef */
const { platform, arch } = process;

let nativeBinding = null;
let localFileExisted = false;
let loadError = null;

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim();
      return require('fs').readFileSync(lddPath, 'utf8').includes('musl');
    } catch {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
  case 'darwin':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync('./cc-validator.darwin-x64.node');
        try {
          nativeBinding = require('./cc-validator.darwin-x64.node');
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        localFileExisted = existsSync('./cc-validator.darwin-arm64.node');
        try {
          nativeBinding = require('./cc-validator.darwin-arm64.node');
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync('./cc-validator.linux-x64-musl.node');
          try {
            nativeBinding = require('./cc-validator.linux-x64-musl.node');
          } catch (e) {
            loadError = e;
          }
        } else {
          localFileExisted = existsSync('./cc-validator.linux-x64-gnu.node');
          try {
            nativeBinding = require('./cc-validator.linux-x64-gnu.node');
          } catch (e) {
            loadError = e;
          }
        }
        break;
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync('./cc-validator.linux-arm64-musl.node');
          try {
            nativeBinding = require('./cc-validator.linux-arm64-musl.node');
          } catch (e) {
            loadError = e;
          }
        } else {
          localFileExisted = existsSync('./cc-validator.linux-arm64-gnu.node');
          try {
            nativeBinding = require('./cc-validator.linux-arm64-gnu.node');
          } catch (e) {
            loadError = e;
          }
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync('./cc-validator.win32-x64-msvc.node');
        try {
          nativeBinding = require('./cc-validator.win32-x64-msvc.node');
        } catch (e) {
          loadError = e;
        }
        break;
      case 'arm64':
        localFileExisted = existsSync('./cc-validator.win32-arm64-msvc.node');
        try {
          nativeBinding = require('./cc-validator.win32-arm64-msvc.node');
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

const { existsSync } = require('fs');

if (!nativeBinding) {
  if (loadError) {
    throw loadError;
  }
  throw new Error(`Failed to load native binding`);
}

const {
  validateCard,
  isValid,
  passesLuhn,
  detectBrand,
  formatCard,
  formatCardWithSeparator,
  stripFormatting,
  maskCard,
  generateTestCard,
  validateCvv,
  validateCvvForBrand,
  validateExpiry,
  parseExpiry,
  validateBatch,
  cvvLengthForBrand,
  validLengthsForBrand,
} = nativeBinding;

module.exports = {
  validateCard,
  isValid,
  passesLuhn,
  detectBrand,
  formatCard,
  formatCardWithSeparator,
  stripFormatting,
  maskCard,
  generateTestCard,
  validateCvv,
  validateCvvForBrand,
  validateExpiry,
  parseExpiry,
  validateBatch,
  cvvLengthForBrand,
  validLengthsForBrand,
};
