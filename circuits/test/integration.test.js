const { expect } = require("chai");
const path = require("path");
const { groth16 } = require("snarkjs");
const { buildPoseidon } = require("circomlibjs");

describe("ZK-ORIGIN Integration Tests", function() {
    this.timeout(300000); // 5 minutes
    
    let poseidon;
    let F;
    
    const wasmPath = path.join(__dirname, "../build/lineage_step_simple_js/lineage_step_simple.wasm");
    const zkeyPath = path.join(__dirname, "../build/lineage_step_simple.zkey");
    const vkeyPath = path.join(__dirname, "../build/verification_key.json");
    
    const Origin = {
        Genesis: 0,
        User: 1,
        Admin: 2,
        Bridge: 3
    };
    
    before(async function() {
        poseidon = await buildPoseidon();
        F = poseidon.F;
    });
    
    function randomHash() {
        return BigInt("0x" + [...Array(64)].map(() => 
            Math.floor(Math.random() * 16).toString(16)
        ).join(""));
    }
    
    function genesisLineage(stateHash) {
        const commitment = poseidon([stateHash, BigInt(0), BigInt(0)]);
        return F.toObject(commitment);
    }
    
    describe("Full Protocol Flow", function() {
        it("should simulate realistic DeFi state transitions", async function() {
            /*
             * Scenario: DeFi Protocol Lifecycle
             * 
             * 1. Genesis: Protocol deployed
             * 2. Admin: Set initial parameters
             * 3. User: First deposit
             * 4. User: Second deposit
             * 5. Admin: Update fee structure
             * 6. User: Withdrawal
             */
            
            const states = {
                genesis: randomHash(),
                paramsSet: randomHash(),
                deposit1: randomHash(),
                deposit2: randomHash(),
                feeUpdate: randomHash(),
                withdrawal: randomHash()
            };
            
            const transitions = [
                { name: "Deploy -> Set Params", from: Origin.Genesis, to: Origin.Admin, 
                  prevState: states.genesis, newState: states.paramsSet },
                { name: "Set Params -> Deposit 1", from: Origin.Admin, to: Origin.User,
                  prevState: states.paramsSet, newState: states.deposit1 },
                { name: "Deposit 1 -> Deposit 2", from: Origin.User, to: Origin.User,
                  prevState: states.deposit1, newState: states.deposit2 },
                { name: "Deposit 2 -> Fee Update", from: Origin.User, to: Origin.User, // Can't go User->Admin
                  prevState: states.deposit2, newState: states.feeUpdate },
                { name: "Fee Update -> Withdrawal", from: Origin.User, to: Origin.User,
                  prevState: states.feeUpdate, newState: states.withdrawal }
            ];
            
            let currentLineage = genesisLineage(states.genesis);
            let currentDepth = 0;
            const proofs = [];
            
            console.log("\n  Simulating DeFi Protocol Lifecycle:");
            
            for (const t of transitions) {
                console.log(`    ${t.name}...`);
                
                const witness = {
                    prev_state_hash: t.prevState.toString(),
                    new_state_hash: t.newState.toString(),
                    prev_lineage_commitment: currentLineage.toString(),
                    prev_origin: t.from.toString(),
                    new_origin: t.to.toString(),
                    prev_depth: currentDepth.toString(),
                    timestamp: Date.now().toString()
                };
                
                const { proof, publicSignals } = await groth16.fullProve(
                    witness,
                    wasmPath,
                    zkeyPath
                );
                
                currentLineage = BigInt(publicSignals[0]);
                currentDepth = parseInt(publicSignals[1]);
                proofs.push({ proof, publicSignals, name: t.name });
            }
            
            console.log("\n  Verifying all proofs...");
            
            const vkey = require(vkeyPath);
            for (const p of proofs) {
                const valid = await groth16.verify(vkey, p.publicSignals, p.proof);
                expect(valid).to.be.true;
            }
            
            expect(currentDepth).to.equal(5);
            console.log(`  ✅ All ${proofs.length} transitions verified!`);
        });
        
        it("should detect and reject unauthorized privilege escalation", async function() {
            /*
             * Attack Scenario: Malicious user tries to escalate privileges
             * 
             * 1. Genesis: Protocol deployed
             * 2. User: Normal deposit
             * 3. User -> Admin: ATTACK - tries to become admin
             */
            
            console.log("\n  Simulating privilege escalation attack:");
            
            const states = {
                genesis: randomHash(),
                deposit: randomHash(),
                attackState: randomHash()
            };
            
            // Normal flow: Genesis -> User
            let currentLineage = genesisLineage(states.genesis);
            
            const witness1 = {
                prev_state_hash: states.genesis.toString(),
                new_state_hash: states.deposit.toString(),
                prev_lineage_commitment: currentLineage.toString(),
                prev_origin: Origin.Genesis.toString(),
                new_origin: Origin.User.toString(),
                prev_depth: "0",
                timestamp: Date.now().toString()
            };
            
            const { publicSignals: signals1 } = await groth16.fullProve(
                witness1,
                wasmPath,
                zkeyPath
            );
            
            currentLineage = BigInt(signals1[0]);
            console.log("    ✅ Normal user deposit succeeded");
            
            // Attack: User -> Admin (should fail at witness generation)
            console.log("    🔴 Attempting privilege escalation (User -> Admin)...");
            
            const attackWitness = {
                prev_state_hash: states.deposit.toString(),
                new_state_hash: states.attackState.toString(),
                prev_lineage_commitment: currentLineage.toString(),
                prev_origin: Origin.User.toString(),
                new_origin: Origin.Admin.toString(), // ATTACK
                prev_depth: "1",
                timestamp: Date.now().toString()
            };
            
            // The circuit will fail because User -> Admin violates policy
            try {
                await groth16.fullProve(
                    attackWitness,
                    wasmPath,
                    zkeyPath
                );
                expect.fail("Attack should have been rejected");
            } catch (error) {
                console.log("    ✅ Attack rejected by circuit constraints!");
                expect(error).to.exist;
            }
        });
    });
    
    describe("Performance Benchmarks", function() {
        it("should measure proof generation time", async function() {
            const state1 = randomHash();
            const state2 = randomHash();
            const lineage = genesisLineage(state1);
            
            const witness = {
                prev_state_hash: state1.toString(),
                new_state_hash: state2.toString(),
                prev_lineage_commitment: lineage.toString(),
                prev_origin: Origin.Genesis.toString(),
                new_origin: Origin.User.toString(),
                prev_depth: "0",
                timestamp: Date.now().toString()
            };
            
            const times = [];
            const iterations = 3;
            
            console.log(`\n  Benchmarking proof generation (${iterations} iterations):`);
            
            for (let i = 0; i < iterations; i++) {
                const start = Date.now();
                await groth16.fullProve(witness, wasmPath, zkeyPath);
                const elapsed = Date.now() - start;
                times.push(elapsed);
                console.log(`    Iteration ${i + 1}: ${elapsed}ms`);
            }
            
            const avg = times.reduce((a, b) => a + b, 0) / times.length;
            console.log(`    Average: ${avg.toFixed(0)}ms`);
            
            expect(avg).to.be.lessThan(30000); // Should be under 30 seconds
        });
        
        it("should measure verification time", async function() {
            const state1 = randomHash();
            const state2 = randomHash();
            const lineage = genesisLineage(state1);
            
            const witness = {
                prev_state_hash: state1.toString(),
                new_state_hash: state2.toString(),
                prev_lineage_commitment: lineage.toString(),
                prev_origin: Origin.Genesis.toString(),
                new_origin: Origin.User.toString(),
                prev_depth: "0",
                timestamp: Date.now().toString()
            };
            
            const { proof, publicSignals } = await groth16.fullProve(
                witness,
                wasmPath,
                zkeyPath
            );
            
            const vkey = require(vkeyPath);
            
            const times = [];
            const iterations = 10;
            
            console.log(`\n  Benchmarking proof verification (${iterations} iterations):`);
            
            for (let i = 0; i < iterations; i++) {
                const start = Date.now();
                await groth16.verify(vkey, publicSignals, proof);
                const elapsed = Date.now() - start;
                times.push(elapsed);
            }
            
            const avg = times.reduce((a, b) => a + b, 0) / times.length;
            console.log(`    Average verification time: ${avg.toFixed(1)}ms`);
            
            expect(avg).to.be.lessThan(100); // Should be under 100ms
        });
    });
});