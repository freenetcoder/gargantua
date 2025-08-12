import { AnonymousAccount } from '../src';

describe('AnonymousAccount', () => {
  describe('generate', () => {
    it('should generate a new anonymous account', () => {
      const account = AnonymousAccount.generate();
      
      expect(account).toBeInstanceOf(AnonymousAccount);
      expect(account.getPrivateKey()).toHaveLength(32);
      expect(account.getPublicKeyBytes()).toHaveLength(32);
    });

    it('should generate different accounts each time', () => {
      const account1 = AnonymousAccount.generate();
      const account2 = AnonymousAccount.generate();
      
      expect(account1.getPrivateKey()).not.toEqual(account2.getPrivateKey());
      expect(account1.getPublicKeyBytes()).not.toEqual(account2.getPublicKeyBytes());
    });
  });

  describe('fromPrivateKey', () => {
    it('should create account from private key', () => {
      const privateKey = new Uint8Array(32);
      privateKey.fill(1);
      
      const account = AnonymousAccount.fromPrivateKey(privateKey);
      
      expect(account.getPrivateKey()).toEqual(privateKey);
    });

    it('should throw error for invalid private key length', () => {
      const invalidKey = new Uint8Array(16);
      
      expect(() => AnonymousAccount.fromPrivateKey(invalidKey)).toThrow();
    });
  });

  describe('fromSeed', () => {
    it('should create account from seed', () => {
      const seed = 'test-seed';
      const account = AnonymousAccount.fromSeed(seed);
      
      expect(account).toBeInstanceOf(AnonymousAccount);
      expect(account.getPrivateKey()).toHaveLength(32);
    });

    it('should create same account from same seed', () => {
      const seed = 'test-seed';
      const account1 = AnonymousAccount.fromSeed(seed);
      const account2 = AnonymousAccount.fromSeed(seed);
      
      expect(account1.getPrivateKey()).toEqual(account2.getPrivateKey());
      expect(account1.getPublicKeyBytes()).toEqual(account2.getPublicKeyBytes());
    });
  });

  describe('signMessage', () => {
    it('should sign a message', () => {
      const account = AnonymousAccount.generate();
      const message = new TextEncoder().encode('test message');
      
      const signature = account.signMessage(message);
      
      expect(signature.challenge).toHaveLength(32);
      expect(signature.response).toHaveLength(32);
    });

    it('should produce different signatures for different messages', () => {
      const account = AnonymousAccount.generate();
      const message1 = new TextEncoder().encode('message 1');
      const message2 = new TextEncoder().encode('message 2');
      
      const sig1 = account.signMessage(message1);
      const sig2 = account.signMessage(message2);
      
      expect(sig1.challenge).not.toEqual(sig2.challenge);
    });
  });

  describe('generateCommitment', () => {
    it('should generate a commitment', () => {
      const account = AnonymousAccount.generate();
      const value = 1000;
      
      const commitment = account.generateCommitment(value);
      
      expect(commitment.commitment).toHaveLength(32);
      expect(commitment.value).toBe(value);
      expect(commitment.randomness).toHaveLength(32);
    });

    it('should generate different commitments for same value', () => {
      const account = AnonymousAccount.generate();
      const value = 1000;
      
      const commitment1 = account.generateCommitment(value);
      const commitment2 = account.generateCommitment(value);
      
      expect(commitment1.commitment).not.toEqual(commitment2.commitment);
      expect(commitment1.randomness).not.toEqual(commitment2.randomness);
    });

    it('should use provided randomness', () => {
      const account = AnonymousAccount.generate();
      const value = 1000;
      const randomness = new Uint8Array(32);
      randomness.fill(42);
      
      const commitment = account.generateCommitment(value, randomness);
      
      expect(commitment.randomness).toEqual(randomness);
    });
  });

  describe('generateOwnershipProof', () => {
    it('should generate ownership proof', async () => {
      const account = AnonymousAccount.generate();
      
      const proof = await account.generateOwnershipProof();
      
      expect(proof.publicKey).toEqual(account.getPublicKeyBytes());
      expect(proof.signature).toBeDefined();
      expect(proof.timestamp).toBeGreaterThan(0);
    });
  });
});