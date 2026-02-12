const { groth16 } = require("snarkjs");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" ZK-ORIGIN Proof Verifier");
    
    const vkeyPath = path.join(__dirname, "../build/verification_key.json");
    const proofsPath = path.join(__dirname, "../build/proofs.json");
    
    if (!fs.existsSync(vkeyPath)) {
        console.error(" Verification key not found. Run setup first.");
        process.exit(1);
    }
    
    if (!fs.existsSync(proofsPath)) {
        console.error(" Proofs file not found. Run prove.js first.");
        process.exit(1);
    }
    
    const vkey = JSON.parse(fs.readFileSync(vkeyPath));
    const proofs = JSON.parse(fs.readFileSync(proofsPath));
    
    console.log(`Found ${proofs.length} proofs to verify\n`);
    
    let allValid = true;
    
    for (let i = 0; i < proofs.length; i++) {
        const { proof, publicSignals } = proofs[i];
        
        console.log(`Proof ${i + 1}:`);
        console.log(`   Lineage commitment: ${publicSignals[0].slice(0, 30)}...`);
        console.log(`   Depth: ${publicSignals[1]}`);
        
        const startTime = Date.now();
        const valid = await groth16.verify(vkey, publicSignals, proof);
        const elapsed = Date.now() - startTime;
        
        console.log(`   Status: ${valid ? ' Valid' : ' Invalid'} (${elapsed}ms)\n`);
        
        if (!valid) allValid = false;
    }
    
    
    console.log(allValid ? " All proofs valid!" : " Some proofs failed!");
}

main().catch(console.error);