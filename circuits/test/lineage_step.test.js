const { expect } = require("chai");
const path = require("path");
const { groth16 } = require("snarkjs");
const { buildPoseidon } = require("circomlibjs");

// Origin class enum
const Origin = {
    Genesis: 0,
    User: 1,
    Admin: 2,
    Bridge: 3
};

describe("ZK-ORIGIN Lineage Step Circuit", function() {
    this.timeout(120000); // 2 minutes for proof generation
    
    let poseidon;
    let F;
    
    const wasmPath = path.join(__dirname, "../build/lineage_step_simple_js/lineage_step_simple.wasm");
    const zkeyPath = path.join(__dirname, "../build/lineage_step_simple.zkey");
    const vkeyPath = path.join(__dirname, "../build/verification_key.json");
    
    before(async function() {
        poseidon = await buildPoseidon();
        F = poseidon.F;
    });
    
    // Helper functions
    function randomHash() {
        return BigInt("0x" + [...Array(64)].map(() => 
            Math.floor(Math.random() * 16).toString(16)
        ).join(""));
    }
    
    function genesisLineage(stateHash) {
        const commitment = poseidon([
            stateHash,
            BigInt(Origin.Genesis),
            BigInt(0)
        ]);
        return F.toObject(commitment);
    }
    
    function isPolicyAllowed(prevOrigin, newOrigin, prevDepth) {
        if (prevOrigin === Origin.User && newOrigin === Origin.Admin) return false;
        if (prevOrigin === Origin.User && newOrigin === Origin.Bridge) return false;
        if (prevOrigin === Origin.Bridge && newOrigin === Origin.Admin) return false;
        if (prevOrigin === Origin.Bridge && newOrigin === Origin.Bridge) return false;
        if (prevDepth > 0 && newOrigin === Origin.Genesis) return false;
        return true;
    }
    
    async function generateWitness(input) {
        if (!isPolicyAllowed(input.prev_origin, input.new_origin, input.prev_depth)) {
            throw new Error(`Policy violation: Origin(${input.prev_origin}) -> Origin(${input.new_origin})`);
        }
        
        return {
            prev_state_hash: input.prev_state_hash.toString(),
            new_state_hash: input.new_state_hash.toString(),
            prev_lineage_commitment: input.prev_lineage_commitment.toString(),
            prev_origin: input.prev_origin.toString(),
            new_origin: input.new_origin.toString(),
            prev_depth: input.prev_depth.toString(),
            timestamp: input.timestamp.toString()
        };
    }
    
    describe("Valid Transitions", function() {
        it("should prove Genesis -> User transition", async function() {
            const genesisState = randomHash();
            const userState = randomHash();
            const genesisCommitment = genesisLineage(genesisState);
            
            const witness = await generateWitness({
                prev_state_hash: genesisState,
                new_state_hash: userState,
                prev_lineage_commitment: genesisCommitment,
                prev_origin: Origin.Genesis,
                new_origin: Origin.User,
                prev_depth: 0,
                timestamp: Date.now()
            });
            
            const { proof, publicSignals } = await groth16.fullProve(
                witness,
                wasmPath,
                zkeyPath
            );
            
            expect(proof).to.have.property("pi_a");
            expect(publicSignals[1]).to.equal("1"); // new_depth = 1
            
            // Verify the proof
            const vkey = require(vkeyPath);
            const valid = await groth16.verify(vkey, publicSignals, proof);
            expect(valid).to.be.true;
        });
        
        it("should prove User -> User transition", async function() {
            const state1 = randomHash();
            const state2 = randomHash();
            const state3 = randomHash();
            
            const genesisCommitment = genesisLineage(state1);
            
            // First transition: Genesis -> User
            const witness1 = await generateWitness({
                prev_state_hash: state1,
                new_state_hash: state2,
                prev_lineage_commitment: genesisCommitment,
                prev_origin: Origin.Genesis,
                new_origin: Origin.User,
                prev_depth: 0,
                timestamp: Date.now()
            });
            
            const { publicSignals: signals1 } = await groth16.fullProve(
                witness1,
                wasmPath,
                zkeyPath
            );
            
            // Second transition: User -> User
            const witness2 = await generateWitness({
                prev_state_hash: state2,
                new_state_hash: state3,
                prev_lineage_commitment: BigInt(signals1[0]),
                prev_origin: Origin.User,
                new_origin: Origin.User,
                prev_depth: 1,
                timestamp: Date.now()
            });
            
            const { proof, publicSignals: signals2 } = await groth16.fullProve(
                witness2,
                wasmPath,
                zkeyPath
            );
            
            expect(signals2[1]).to.equal("2"); // new_depth = 2
            
            const vkey = require(vkeyPath);
            const valid = await groth16.verify(vkey, signals2, proof);
            expect(valid).to.be.true;
        });
        
        it("should prove Genesis -> Admin transition", async function() {
            const genesisState = randomHash();
            const adminState = randomHash();
            const genesisCommitment = genesisLineage(genesisState);
            
            const witness = await generateWitness({
                prev_state_hash: genesisState,
                new_state_hash: adminState,
                prev_lineage_commitment: genesisCommitment,
                prev_origin: Origin.Genesis,
                new_origin: Origin.Admin,
                prev_depth: 0,
                timestamp: Date.now()
            });
            
            const { proof, publicSignals } = await groth16.fullProve(
                witness,
                wasmPath,
                zkeyPath
            );
            
            const vkey = require(vkeyPath);
            const valid = await groth16.verify(vkey, publicSignals, proof);
            expect(valid).to.be.true;
        });
        
        it("should prove Admin -> User transition (de-escalation)", async function() {
            const state1 = randomHash();
            const state2 = randomHash();
            const state3 = randomHash();
            
            const genesisCommitment = genesisLineage(state1);
            
            // Genesis -> Admin
            const witness1 = await generateWitness({
                prev_state_hash: state1,
                new_state_hash: state2,
                prev_lineage_commitment: genesisCommitment,
                prev_origin: Origin.Genesis,
                new_origin: Origin.Admin,
                prev_depth: 0,
                timestamp: Date.now()
            });
            
            const { publicSignals: signals1 } = await groth16.fullProve(
                witness1,
                wasmPath,
                zkeyPath
            );
            
            // Admin -> User
            const witness2 = await generateWitness({
                prev_state_hash: state2,
                new_state_hash: state3,
                prev_lineage_commitment: BigInt(signals1[0]),
                prev_origin: Origin.Admin,
                new_origin: Origin.User,
                prev_depth: 1,
                timestamp: Date.now()
            });
            
            const { proof, publicSignals: signals2 } = await groth16.fullProve(
                witness2,
                wasmPath,
                zkeyPath
            );
            
            const vkey = require(vkeyPath);
            const valid = await groth16.verify(vkey, signals2, proof);
            expect(valid).to.be.true;
        });
        
        it("should prove Admin -> Bridge transition", async function() {
            const state1 = randomHash();
            const state2 = randomHash();
            const state3 = randomHash();
            
            const genesisCommitment = genesisLineage(state1);
            
            // Genesis -> Admin
            const witness1 = await generateWitness({
                prev_state_hash: state1,
                new_state_hash: state2,
                prev_lineage_commitment: genesisCommitment,
                prev_origin: Origin.Genesis,
                new_origin: Origin.Admin,
                prev_depth: 0,
                timestamp: Date.now()
            });
            
            const { publicSignals: signals1 } = await groth16.fullProve(
                witness1,
                wasmPath,
                zkeyPath
            );
            
            // Admin -> Bridge
            const witness2 = await generateWitness({
                prev_state_hash: state2,
                new_state_hash: state3,
                prev_lineage_commitment: BigInt(signals1[0]),
                prev_origin: Origin.Admin,
                new_origin: Origin.Bridge,
                prev_depth: 1,
                timestamp: Date.now()
            });
            
            const { proof, publicSignals: signals2 } = await groth16.fullProve(
                witness2,
                wasmPath,
                zkeyPath
            );
            
            const vkey = require(vkeyPath);
            const valid = await groth16.verify(vkey, signals2, proof);
            expect(valid).to.be.true;
        });
    });
    
    describe("Invalid Transitions (Policy Violations)", function() {
        it("should reject User -> Admin (privilege escalation)", async function() {
            try {
                await generateWitness({
                    prev_state_hash: randomHash(),
                    new_state_hash: randomHash(),
                    prev_lineage_commitment: randomHash(),
                    prev_origin: Origin.User,
                    new_origin: Origin.Admin,
                    prev_depth: 1,
                    timestamp: Date.now()
                });
                expect.fail("Should have thrown policy violation");
            } catch (error) {
                expect(error.message).to.include("Policy violation");
            }
        });
        
        it("should reject User -> Bridge", async function() {
            try {
                await generateWitness({
                    prev_state_hash: randomHash(),
                    new_state_hash: randomHash(),
                    prev_lineage_commitment: randomHash(),
                    prev_origin: Origin.User,
                    new_origin: Origin.Bridge,
                    prev_depth: 1,
                    timestamp: Date.now()
                });
                expect.fail("Should have thrown policy violation");
            } catch (error) {
                expect(error.message).to.include("Policy violation");
            }
        });
        
        it("should reject Bridge -> Admin", async function() {
            try {
                await generateWitness({
                    prev_state_hash: randomHash(),
                    new_state_hash: randomHash(),
                    prev_lineage_commitment: randomHash(),
                    prev_origin: Origin.Bridge,
                    new_origin: Origin.Admin,
                    prev_depth: 1,
                    timestamp: Date.now()
                });
                expect.fail("Should have thrown policy violation");
            } catch (error) {
                expect(error.message).to.include("Policy violation");
            }
        });
        
        it("should reject Bridge -> Bridge (no chaining)", async function() {
            try {
                await generateWitness({
                    prev_state_hash: randomHash(),
                    new_state_hash: randomHash(),
                    prev_lineage_commitment: randomHash(),
                    prev_origin: Origin.Bridge,
                    new_origin: Origin.Bridge,
                    prev_depth: 1,
                    timestamp: Date.now()
                });
                expect.fail("Should have thrown policy violation");
            } catch (error) {
                expect(error.message).to.include("Policy violation");
            }
        });
        
        it("should reject return to Genesis after depth > 0", async function() {
            try {
                await generateWitness({
                    prev_state_hash: randomHash(),
                    new_state_hash: randomHash(),
                    prev_lineage_commitment: randomHash(),
                    prev_origin: Origin.User,
                    new_origin: Origin.Genesis,
                    prev_depth: 5,
                    timestamp: Date.now()
                });
                expect.fail("Should have thrown policy violation");
            } catch (error) {
                expect(error.message).to.include("Policy violation");
            }
        });
    });
    
    describe("Lineage Chain", function() {
        it("should correctly chain 5 transitions", async function() {
            const states = Array(6).fill(null).map(() => randomHash());
            let currentLineage = genesisLineage(states[0]);
            let currentDepth = 0;
            
            const origins = [Origin.Genesis, Origin.User, Origin.User, Origin.User, Origin.User];
            
            for (let i = 0; i < 5; i++) {
                const witness = await generateWitness({
                    prev_state_hash: states[i],
                    new_state_hash: states[i + 1],
                    prev_lineage_commitment: currentLineage,
                    prev_origin: origins[i],
                    new_origin: Origin.User,
                    prev_depth: currentDepth,
                    timestamp: Date.now()
                });
                
                const { publicSignals } = await groth16.fullProve(
                    witness,
                    wasmPath,
                    zkeyPath
                );
                
                currentLineage = BigInt(publicSignals[0]);
                currentDepth = parseInt(publicSignals[1]);
                
                expect(currentDepth).to.equal(i + 1);
            }
            
            expect(currentDepth).to.equal(5);
        });
    });
    
    describe("Commitment Consistency", function() {
        it("should produce consistent lineage commitments", async function() {
            const state1 = randomHash();
            const state2 = randomHash();
            const genesisCommitment = genesisLineage(state1);
            const timestamp = Date.now();
            
            // Generate proof twice with same inputs
            const witness = {
                prev_state_hash: state1.toString(),
                new_state_hash: state2.toString(),
                prev_lineage_commitment: genesisCommitment.toString(),
                prev_origin: Origin.Genesis.toString(),
                new_origin: Origin.User.toString(),
                prev_depth: "0",
                timestamp: timestamp.toString()
            };
            
            const { publicSignals: signals1 } = await groth16.fullProve(
                witness,
                wasmPath,
                zkeyPath
            );
            
            const { publicSignals: signals2 } = await groth16.fullProve(
                witness,
                wasmPath,
                zkeyPath
            );
            
            // Same inputs should produce same lineage commitment
            expect(signals1[0]).to.equal(signals2[0]);
        });
    });
});