/**
 * Complete E2E: Generate witness → proof → submit → verify
 * 
 * This orchestrates the entire flow from state transition to verified state
 */

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(`\n${'═'.repeat(60)}`);
    console.log("🚀 ZK-ORIGIN E2E PROOF PIPELINE");
    console.log(`${'═'.repeat(60)}\n`);
    
    const steps = [
        {
            name: "1️⃣  Generate Witness",
            cmd: "cargo run --release --manifest-path=../prover/Cargo.toml --bin witness_gen",
            output: "../prover/witness.json",
        },
        {
            name: "2️⃣  Generate Proof",
            cmd: "cd circuits && snarkjs groth16 prove build/main_final.zkey build/witness.wtns proof.json public.json",
            output: "circuits/proof.json",
        },
        {
            name: "3️⃣  Submit to Contract",
            cmd: "node scripts/submit_proof.js",
            output: "proof_submission_result.json",
        },
        {
            name: "4️⃣  Verify State",
            cmd: "node scripts/verify_state.js",
            output: "state_verification.json",
        },
    ];
    
    let success = true;
    
    for (const step of steps) {
        console.log(`\n${step.name}`);
        console.log(`${'─'.repeat(40)}`);
        
        try {
            console.log(`⏳ Running: ${step.cmd}\n`);
            const output = execSync(step.cmd, { encoding: "utf8", stdio: "inherit" });
            
            if (fs.existsSync(step.output)) {
                console.log(`✅ Completed - Output: ${step.output}`);
            } else {
                console.warn(`⚠️  Output file not found: ${step.output}`);
            }
        } catch (error) {
            console.error(`❌ Failed with error:`, error.message);
            success = false;
            break;
        }
    }
    
    console.log(`\n${'═'.repeat(60)}`);
    if (success) {
        console.log("✅ E2E PIPELINE COMPLETED SUCCESSFULLY");
    } else {
        console.log("❌ E2E PIPELINE FAILED");
    }
    console.log(`${'═'.repeat(60)}\n`);
}

main().catch(console.error);