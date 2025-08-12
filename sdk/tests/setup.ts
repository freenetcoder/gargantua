// Jest setup file
import { TextEncoder, TextDecoder } from 'util';

// Polyfill for Node.js environment
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder as any;

// Mock crypto for Node.js environment
Object.defineProperty(global, 'crypto', {
  value: {
    getRandomValues: (arr: any) => {
      const crypto = require('crypto');
      const bytes = crypto.randomBytes(arr.length);
      arr.set(bytes);
      return arr;
    },
  },
});

// Increase timeout for integration tests
jest.setTimeout(30000);