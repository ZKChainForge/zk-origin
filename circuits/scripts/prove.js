const { groth16 } = require("snarkjs");
const fs = require("fs");
const path = require("path");
const { buildPoseidon } = require("circomlibjs");

// Origin class enum
const Origin = {
    Genesis: 0,
    User: 1,
    Admin: 2,
    Bridge: 3
};

// Policy check (mirrors circuit logic)
function isPolicyAllowed(prevOrigin, newOrigin, prevDepth) {
    // User can't escalate to Admin
    if (prevOrigin === Origin.User && newOrigin === Origin.Admin) return false;
    
    // User can't create Bridge transitions
    if (prevOrigin === Origin.User && newOrigin === Origin.Bridge) return false;
    
    // Bridge can't escalate to Admin
    if (prevOrigin === Origin.Bridge && newOrigin === Origin.Admin) return false;
    
    // Bridge can't chain to Bridge
    if (prevOrigin === Origin.Bridge && newOrigin === Origin.Bridge) return false;
    
    // Can't return to Genesis after starting
    if (prevDepth > 0 && newOrigin === Origin.Genesis) return false;
    
    return true;
}

async function main() {
    console.log("ZK-ORIGIN Proof Generator");
    
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    
    const wasmPath = path.join(__dirname, "../build/lineage_step_simple_js/lineage_step_simple.wasm");
    const zkeyPath = path.join(__dirname, "../build/lineage_step_simple.zkey");
    
    // Check files exist
    if (!fs.existsSync(wasmPath)) {
        console.error(" WASM file not found. Run ./scripts/compile.sh first");
        process.exit(1);
    }
    if (!fs.existsSync(zkeyPath)) {
        console.error("ZKey file not found. Run ./scripts/setup.sh first");
        process.exit(1);
    }
    
    // Genesis lineage commitment
    function genesisLineage(stateHash) {
        const commitment = poseidon([
            stateHash,
            BigInt(Origin.Genesis),
            BigInt(0)
        ]);
        return F.toObject(commitment);
    }
    
    // Generate witness for a transition
    async function generateWitness(input) {
        if (!isPolicyAllowed(input.prev_origin, input.new_origin, input.prev_depth)) {
            throw new Error(`Policy violation: ${input.prev_origin} -> ${input.new_origin}`);
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
    
    // Example states
    const states = [
        BigInt("0x1111111111111111111111111111111111111111111111111111111111111111"),
        BigInt("0x2222222222222222222222222222222222222222222222222222222222222222"),
        BigInt("0x3333333333333333333333333333333333333333333333333333333333333333"),
        BigInt("0x4444444444444444444444444444444444444444444444444444444444444444")
    ];
    
    console.log(" Creating lineage chain:\n");
    console.log("   Genesis -> User -> User -> Admin\n");
    
    let currentLineage = genesisLineage(states[0]);
    let proofs = [];
    
    const transitions = [
        { from: Origin.Genesis, to: Origin.User, stateIdx: 0 },
        { from: Origin.User, to: Origin.User, stateIdx: 1 },
        { from: Origin.User, to: Origin.User, stateIdx: 2 }  // Note: Can't do User->Admin
    ];
    
    for (let i = 0; i < transitions.length; i++) {
        const t = transitions[i];
        console.log(`Step ${i + 1}: ${Object.keys(Origin)[t.from]} -> ${Object.keys(Origin)[t.to]}`);
        
        const witness = await generateWitness({
            prev_state_hash: states[t.stateIdx],
            new_state_hash: states[t.stateIdx + 1],
            prev_lineage_commitment: currentLineage,
            prev_origin: t.from,
            new_origin: t.to,
            prev_depth: i,
            timestamp: Date.now()
        });
        
        console.log("   Generating proof...");
        const startTime = Date.now();
        
        const { proof, publicSignals } = await groth16.fullProve(
            witness,
            wasmPath,
            zkeyPath
        );
        
        const elapsed = Date.now() - startTime;
        console.log(`   Proof generated in ${elapsed}ms`);
        console.log(`   New lineage: ${publicSignals[0].slice(0, 20)}...`);
        console.log(`   New depth: ${publicSignals[1]}\n`);
        
        currentLineage = BigInt(publicSignals[0]);
        proofs.push({ proof, publicSignals });
    }
    
    // Save proofs
    const outputPath = path.join(__dirname, "../build/proofs.json");
    fs.writeFileSync(outputPath, JSON.stringify(proofs, null, 2));
    console.log(`Proofs saved to: ${outputPath}`);
    
    // Verify proofs
    console.log("\nVerifying proofs locally...\n");
    const vkey = JSON.parse(fs.readFileSync(
        path.join(__dirname, "../build/verification_key.json")
    ));
    
    for (let i = 0; i < proofs.length; i++) {
        const valid = await groth16.verify(vkey, proofs[i].publicSignals, proofs[i].proof);
        console.log(`   Proof ${i + 1}: ${valid ? 'Valid' : ' Invalid'}`);
    }
    
    console.log("\n Demo complete!");
}

main().catch(err => {
    console.error(" Error:", err.message);
    process.exit(1);
});