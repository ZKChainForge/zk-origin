// contracts/test/integration.test.ts

import { expect } from 'chai';
import { ethers } from 'hardhat';
import * as snarkjs from 'snarkjs';
import path from 'path';

describe('ZK-ORIGIN Full Integration', function() {
    this.timeout(120000); // 2 minutes for proof generation
    
    let verifier: any;
    let lineageVerifier: any;
    let owner: any;
    let user: any;
    
    const CIRCUIT_WASM = path.join(__dirname, '../../circuits/build/lineage_step.wasm');
    const CIRCUIT_ZKEY = path.join(__dirname, '../../circuits/build/lineage_step_final.zkey');
    
    before(async function() {
        [owner, user] = await ethers.getSigners();
        
        const Verifier = await ethers.getContractFactory('Groth16Verifier');
        verifier = await Verifier.deploy();
        
        const LineageVerifier = await ethers.getContractFactory('LineageVerifier');
        lineageVerifier = await LineageVerifier.deploy(await verifier.getAddress());
    });
    
    describe('Genesis Setup', function() {
        it('should set genesis state', async function() {
            const genesisState = ethers.keccak256(ethers.toUtf8Bytes('genesis'));
            const genesisLineage = ethers.keccak256(ethers.toUtf8Bytes('lineage0'));
            
            await lineageVerifier.setGenesis(genesisState, genesisLineage);
            
            expect(await lineageVerifier.genesisInitialized()).to.be.true;
            expect(await lineageVerifier.hasVerifiedLineage(genesisState)).to.be.true;
        });
        
        it('should reject duplicate genesis', async function() {
            const newGenesis = ethers.keccak256(ethers.toUtf8Bytes('genesis2'));
            await expect(
                lineageVerifier.setGenesis(newGenesis, newGenesis)
            ).to.be.revertedWithCustomError(lineageVerifier, 'GenesisAlreadySet');
        });
    });
    
    describe('Proof Verification', function() {
        it('should verify valid proof', async function() {
            // Get current genesis info
            const genesisState = await lineageVerifier.genesisStateHash();
            const genesisLineage = await lineageVerifier.genesisLineageCommitment();
            
            // Prepare circuit input
            const newState = ethers.keccak256(ethers.toUtf8Bytes('state1'));
            const policyRoot = ethers.keccak256(ethers.toUtf8Bytes('policy'));
            
            const input = {
                prev_lineage_commitment: BigInt(genesisLineage).toString(),
                new_state_hash: BigInt(newState).toString(),
                prev_state_hash: BigInt(genesisState).toString(),
                policy_root: BigInt(policyRoot).toString(),
                prev_origin: 0,
                new_origin: 1,
                prev_depth: 0,
                timestamp: Math.floor(Date.now() / 1000),
                policy_proof: Array(4).fill('0'),
                policy_indices: Array(4).fill(0)
            };
            
            // Generate proof
            const { proof, publicSignals } = await snarkjs.groth16.fullProve(
                input,
                CIRCUIT_WASM,
                CIRCUIT_ZKEY
            );
            
            // Format for Solidity
            const pA = [proof.pi_a[0], proof.pi_a[1]];
            const pB = [
                [proof.pi_b[0][1], proof.pi_b[0][0]],
                [proof.pi_b[1][1], proof.pi_b[1][0]]
            ];
            const pC = [proof.pi_c[0], proof.pi_c[1]];
            
            // Submit proof
            const tx = await lineageVerifier.verifyLineage(pA, pB, pC, publicSignals);
            const receipt = await tx.wait();
            
            expect(receipt.status).to.equal(1);
            
            // Verify state was recorded
            expect(await lineageVerifier.hasVerifiedLineage(newState)).to.be.true;
        });
        
        it('should reject invalid proof', async function() {
            const invalidProof = {
                pA: ['1', '2'],
                pB: [['1', '2'], ['3', '4']],
                pC: ['1', '2']
            };
            
            await expect(
                lineageVerifier.verifyLineage(
                    invalidProof.pA,
                    invalidProof.pB,
                    invalidProof.pC,
                    ['0', '0', '0', '0', '0']
                )
            ).to.be.revertedWithCustomError(lineageVerifier, 'InvalidProof');
        });
    });
    
    describe('Lineage Chain', function() {
        it('should build multi-step lineage', async function() {
            const states = [];
            let prevState = await lineageVerifier.genesisStateHash();
            let prevLineage = await lineageVerifier.genesisLineageCommitment();
            let depth = 0;
            
            for (let i = 0; i < 5; i++) {
                const newState = ethers.keccak256(ethers.toUtf8Bytes(`chain_state_${i}`));
                
                const input = {
                    prev_lineage_commitment: BigInt(prevLineage).toString(),
                    new_state_hash: BigInt(newState).toString(),
                    prev_state_hash: BigInt(prevState).toString(),
                    policy_root: BigInt(ethers.ZeroHash).toString(),
                    prev_origin: i === 0 ? 0 : 1,
                    new_origin: 1,
                    prev_depth: depth,
                    timestamp: Math.floor(Date.now() / 1000),
                    policy_proof: Array(4).fill('0'),
                    policy_indices: Array(4).fill(0)
                };
                
                const { proof, publicSignals } = await snarkjs.groth16.fullProve(
                    input,
                    CIRCUIT_WASM,
                    CIRCUIT_ZKEY
                );
                
                const pA = [proof.pi_a[0], proof.pi_a[1]];
                const pB = [
                    [proof.pi_b[0][1], proof.pi_b[0][0]],
                    [proof.pi_b[1][1], proof.pi_b[1][0]]
                ];
                const pC = [proof.pi_c[0], proof.pi_c[1]];
                
                await lineageVerifier.verifyLineage(pA, pB, pC, publicSignals);
                
                prevState = newState;
                prevLineage = await lineageVerifier.getLineage(newState);
                depth++;
                
                states.push({
                    hash: newState,
                    lineage: prevLineage,
                    depth
                });
            }
            
            // Verify chain integrity
            const finalDepth = await lineageVerifier.getDepth(states[4].hash);
            expect(finalDepth).to.equal(5);
            
            for (const state of states) {
                expect(await lineageVerifier.hasVerifiedLineage(state.hash)).to.be.true;
            }
        });
    });
});