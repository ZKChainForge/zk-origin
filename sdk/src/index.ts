// sdk/src/index.ts

import { ethers } from 'ethers';
import * as snarkjs from 'snarkjs';

export interface ZKOriginConfig {
    rpcUrl: string;
    lineageVerifierAddress: string;
    circuitWasmPath: string;
    circuitZkeyPath: string;
}

export interface TransitionInput {
    prevStateHash: string;
    newStateHash: string;
    prevLineageCommitment: string;
    policyRoot: string;
    prevOrigin: number;
    newOrigin: number;
    prevDepth: number;
    timestamp: number;
    policyProof: string[];
    policyIndices: number[];
}

export interface LineageProof {
    pA: [string, string];
    pB: [[string, string], [string, string]];
    pC: [string, string];
    publicSignals: string[];
}

export class ZKOriginClient {
    private provider: ethers.Provider;
    private contract: ethers.Contract;
    private config: ZKOriginConfig;
    
    constructor(config: ZKOriginConfig) {
        this.config = config;
        this.provider = new ethers.JsonRpcProvider(config.rpcUrl);
        
        const abi = [
            'function verifyLineage(uint256[2] pA, uint256[2][2] pB, uint256[2] pC, uint256[5] publicSignals) external returns (bool)',
            'function hasVerifiedLineage(bytes32 stateHash) external view returns (bool)',
            'function getLineage(bytes32 stateHash) external view returns (bytes32)',
            'function getDepth(bytes32 stateHash) external view returns (uint256)',
            'event LineageVerified(bytes32 indexed prevStateHash, bytes32 indexed newStateHash, bytes32 lineageCommitment, uint256 depth)'
        ];
        
        this.contract = new ethers.Contract(config.lineageVerifierAddress, abi, this.provider);
    }
    
    async generateProof(input: TransitionInput): Promise<LineageProof> {
        const circuitInput = {
            prev_lineage_commitment: BigInt(input.prevLineageCommitment).toString(),
            prev_state_hash: BigInt(input.prevStateHash).toString(),
            new_state_hash: BigInt(input.newStateHash).toString(),
            policy_root: BigInt(input.policyRoot).toString(),
            prev_origin: input.prevOrigin,
            new_origin: input.newOrigin,
            prev_depth: input.prevDepth,
            timestamp: input.timestamp,
            policy_proof: input.policyProof.map(p => BigInt(p).toString()),
            policy_indices: input.policyIndices
        };
        
        const { proof, publicSignals } = await snarkjs.groth16.fullProve(
            circuitInput,
            this.config.circuitWasmPath,
            this.config.circuitZkeyPath
        );
        
        return {
            pA: [proof.pi_a[0], proof.pi_a[1]],
            pB: [
                [proof.pi_b[0][1], proof.pi_b[0][0]],
                [proof.pi_b[1][1], proof.pi_b[1][0]]
            ],
            pC: [proof.pi_c[0], proof.pi_c[1]],
            publicSignals
        };
    }
    
    async submitProof(proof: LineageProof, signer: ethers.Signer): Promise<ethers.TransactionReceipt> {
        const connectedContract = this.contract.connect(signer);
        
        const tx = await connectedContract.verifyLineage(
            proof.pA,
            proof.pB,
            proof.pC,
            proof.publicSignals
        );
        
        return await tx.wait();
    }
    
    async isVerified(stateHash: string): Promise<boolean> {
        return await this.contract.hasVerifiedLineage(stateHash);
    }
    
    async getLineage(stateHash: string): Promise<string> {
        return await this.contract.getLineage(stateHash);
    }
    
    async getDepth(stateHash: string): Promise<number> {
        const depth = await this.contract.getDepth(stateHash);
        return Number(depth);
    }
    
    async getFullState(stateHash: string): Promise<{
        lineageCommitment: string;
        depth: number;
        verified: boolean;
    }> {
        const [lineage, depth, verified] = await Promise.all([
            this.getLineage(stateHash),
            this.getDepth(stateHash),
            this.isVerified(stateHash)
        ]);
        
        return { lineageCommitment: lineage, depth, verified };
    }
}

// Export convenient factory
export function createClient(config: ZKOriginConfig): ZKOriginClient {
    return new ZKOriginClient(config);
}