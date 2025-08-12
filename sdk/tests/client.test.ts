import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { GargantuaClient, AnonymousAccount } from '../src';

// Mock Solana connection
jest.mock('@solana/web3.js', () => ({
  ...jest.requireActual('@solana/web3.js'),
  Connection: jest.fn().mockImplementation(() => ({
    getAccountInfo: jest.fn(),
    getRecentBlockhash: jest.fn().mockResolvedValue({
      feeCalculator: { lamportsPerSignature: 5000 }
    }),
  })),
  sendAndConfirmTransaction: jest.fn().mockResolvedValue('mock-signature'),
}));

describe('GargantuaClient', () => {
  let client: GargantuaClient;
  let mockConnection: jest.Mocked<Connection>;

  beforeEach(() => {
    mockConnection = new Connection('https://api.devnet.solana.com') as jest.Mocked<Connection>;
    client = new GargantuaClient(mockConnection);
  });

  describe('constructor', () => {
    it('should create client with default config', () => {
      expect(client).toBeInstanceOf(GargantuaClient);
    });

    it('should create client with custom config', () => {
      const customClient = new GargantuaClient(mockConnection, {
        commitment: 'finalized',
        maxRetries: 5,
      });
      expect(customClient).toBeInstanceOf(GargantuaClient);
    });
  });

  describe('registerAccount', () => {
    it('should register a new anonymous account', async () => {
      const payer = Keypair.generate();
      
      // Mock successful registration
      mockConnection.getAccountInfo.mockResolvedValue(null);
      
      const account = await client.registerAccount(payer);
      
      expect(account).toBeInstanceOf(AnonymousAccount);
      expect(account.getAccountAddress()).toBeDefined();
      expect(account.getPendingAddress()).toBeDefined();
    });
  });

  describe('getGlobalState', () => {
    it('should return null when global state does not exist', async () => {
      mockConnection.getAccountInfo.mockResolvedValue(null);
      
      const globalState = await client.getGlobalState();
      
      expect(globalState).toBeNull();
    });

    it('should return global state when it exists', async () => {
      const mockAccountData = Buffer.alloc(96);
      mockConnection.getAccountInfo.mockResolvedValue({
        data: mockAccountData,
        executable: false,
        lamports: 1000000,
        owner: new PublicKey('11111111111111111111111111111111'),
        rentEpoch: 0,
      });
      
      const globalState = await client.getGlobalState();
      
      expect(globalState).toBeDefined();
      expect(globalState?.authority).toBeInstanceOf(PublicKey);
    });
  });

  describe('estimateFee', () => {
    it('should estimate transaction fee', async () => {
      const mockInstruction = {
        keys: [],
        programId: new PublicKey('11111111111111111111111111111111'),
        data: Buffer.alloc(0),
      };
      
      const fee = await client.estimateFee(mockInstruction);
      
      expect(typeof fee).toBe('number');
      expect(fee).toBeGreaterThan(0);
    });
  });
});