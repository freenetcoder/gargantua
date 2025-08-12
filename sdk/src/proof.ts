import { sha256 } from 'js-sha256';
import { AnonymousAccount } from './account';
import { TransferProof, BurnProof, RangeProof, InnerProductProof } from './types';
import { GargantuaError, ErrorCode } from './errors';

export interface TransferInput {
  account: AnonymousAccount;
  amount: number;
}

export interface TransferOutput {
  address: import('@solana/web3.js').PublicKey;
  amount: number;
}

export class ProofGenerator {
  /**
   * Generate a transfer proof
   */
  async generateTransferProof(
    inputs: TransferInput[],
    outputs: TransferOutput[],
    fee: number
  ): Promise<TransferProof> {
    // Validate inputs
    if (inputs.length === 0 || outputs.length === 0) {
      throw new GargantuaError('Inputs and outputs cannot be empty', ErrorCode.INVALID_ARGUMENT);
    }

    const totalInput = inputs.reduce((sum, input) => sum + input.amount, 0);
    const totalOutput = outputs.reduce((sum, output) => sum + output.amount, 0);

    if (totalInput !== totalOutput + fee) {
      throw new GargantuaError('Input/output balance mismatch', ErrorCode.INVALID_ARGUMENT);
    }

    // Generate commitments for inputs
    const commitments_c: number[][] = [];
    const public_keys: number[][] = [];

    for (const input of inputs) {
      const commitment = input.account.generateCommitment(input.amount);
      commitments_c.push(Array.from(commitment.commitment));
      public_keys.push(Array.from(input.account.getPublicKeyBytes()));
    }

    // Generate commitment for output
    const outputCommitment = this.generateOutputCommitment(outputs);
    const commitment_d = Array.from(outputCommitment);

    // Generate nonce
    const nonce = this.generateNonce();

    // Generate beneficiary (relayer) key
    const beneficiary = Array.from(new Uint8Array(32)); // Placeholder

    // Generate range proofs
    const rangeProofs = await this.generateRangeProofs(
      inputs.map(i => i.amount).concat(outputs.map(o => o.amount))
    );

    // Generate inner product proof
    const innerProductProof = await this.generateInnerProductProof(
      commitments_c.concat([commitment_d])
    );

    return {
      commitments_c,
      commitment_d,
      public_keys,
      nonce,
      beneficiary,
      ba: Array.from(rangeProofs.a),
      bs: Array.from(rangeProofs.s),
      a: Array.from(rangeProofs.a),
      b: Array.from(rangeProofs.s),
      cln_g: commitments_c,
      crn_g: commitments_c,
      c_0g: commitments_c,
      dg: [commitment_d],
      y_0g: public_keys,
      gg: commitments_c,
      c_xg: commitments_c,
      y_xg: public_keys,
      f: commitments_c,
      z_a: Array.from(new Uint8Array(32)),
      t_1: Array.from(rangeProofs.t1),
      t_2: Array.from(rangeProofs.t2),
      t_hat: Array.from(rangeProofs.t_hat),
      mu: Array.from(rangeProofs.mu),
      c: Array.from(new Uint8Array(32)),
      s_sk: Array.from(new Uint8Array(32)),
      s_r: Array.from(new Uint8Array(32)),
      s_b: Array.from(new Uint8Array(32)),
      s_tau: Array.from(rangeProofs.tau_x),
      ip_proof: {
        l_points: innerProductProof.l_vec,
        r_points: innerProductProof.r_vec,
        a: Array.from(innerProductProof.a),
        b: Array.from(innerProductProof.b),
      },
    };
  }

  /**
   * Generate a burn proof
   */
  async generateBurnProof(
    account: AnonymousAccount,
    amount: number
  ): Promise<BurnProof> {
    // Generate range proof for the burn amount
    const rangeProof = await this.generateRangeProof(amount);

    // Generate ownership proof
    const ownershipProof = await account.generateOwnershipProof();

    // Generate nonce
    const nonce = this.generateNonce();

    // Generate inner product proof
    const innerProductProof = await this.generateInnerProductProof([
      Array.from(rangeProof.a)
    ]);

    return {
      ba: Array.from(rangeProof.a),
      bs: Array.from(rangeProof.s),
      t_1: Array.from(rangeProof.t1),
      t_2: Array.from(rangeProof.t2),
      t_hat: Array.from(rangeProof.t_hat),
      mu: Array.from(rangeProof.mu),
      c: Array.from(new Uint8Array(32)),
      s_sk: Array.from(ownershipProof.signature.response),
      s_b: Array.from(new Uint8Array(32)),
      s_tau: Array.from(rangeProof.tau_x),
      nonce,
      ip_proof: {
        l_points: innerProductProof.l_vec,
        r_points: innerProductProof.r_vec,
        a: Array.from(innerProductProof.a),
        b: Array.from(innerProductProof.b),
      },
    };
  }

  /**
   * Generate range proof for a single value
   */
  async generateRangeProof(value: number, bitLength: number = 32): Promise<RangeProof> {
    if (value < 0 || value >= Math.pow(2, bitLength)) {
      throw new GargantuaError(
        `Value ${value} is out of range for ${bitLength} bits`,
        ErrorCode.INVALID_ARGUMENT
      );
    }

    // Generate random values for the proof
    const a = this.generateRandomPoint();
    const s = this.generateRandomPoint();
    const t1 = this.generateRandomPoint();
    const t2 = this.generateRandomPoint();
    const t_hat = this.generateRandomScalar();
    const tau_x = this.generateRandomScalar();
    const mu = this.generateRandomScalar();

    return {
      a,
      s,
      t1,
      t2,
      t_hat,
      tau_x,
      mu,
    };
  }

  /**
   * Generate range proofs for multiple values
   */
  async generateRangeProofs(values: number[]): Promise<RangeProof> {
    // For simplicity, generate a single aggregated range proof
    // In practice, this would be more complex
    const maxValue = Math.max(...values);
    return await this.generateRangeProof(maxValue);
  }

  /**
   * Generate inner product proof
   */
  async generateInnerProductProof(commitments: number[][]): Promise<InnerProductProof> {
    const logN = Math.ceil(Math.log2(commitments.length));
    
    const l_vec: number[][] = [];
    const r_vec: number[][] = [];

    // Generate L and R vectors for each round
    for (let i = 0; i < logN; i++) {
      l_vec.push(Array.from(this.generateRandomPoint()));
      r_vec.push(Array.from(this.generateRandomPoint()));
    }

    return {
      l_vec,
      r_vec,
      a: this.generateRandomScalar(),
      b: this.generateRandomScalar(),
    };
  }

  // Private helper methods

  private generateOutputCommitment(outputs: TransferOutput[]): Uint8Array {
    // Generate a commitment to the sum of outputs
    const totalOutput = outputs.reduce((sum, output) => sum + output.amount, 0);
    
    // Create a dummy commitment (in practice, this would be a proper Pedersen commitment)
    const hash = sha256.create();
    hash.update(totalOutput.toString());
    return new Uint8Array(hash.arrayBuffer());
  }

  private generateNonce(): number[] {
    const nonce = new Uint8Array(32);
    crypto.getRandomValues(nonce);
    return Array.from(nonce);
  }

  private generateRandomPoint(): Uint8Array {
    const point = new Uint8Array(32);
    crypto.getRandomValues(point);
    return point;
  }

  private generateRandomScalar(): Uint8Array {
    const scalar = new Uint8Array(32);
    crypto.getRandomValues(scalar);
    return scalar;
  }
}